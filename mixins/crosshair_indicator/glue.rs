// Glue between Horizon's `Game` and `mod_crosshair_indicator`
use crate::game::Game;
use crate::game::mod_crosshair_indicator as m;
use crate::game::native_hooks::NativeMod;
use horizon_client::extras_hud::Corners;
use horizon_client::hud::HudBatch;

pub(super) fn on_hud(game: &mut Game<'_>, batch: &mut HudBatch, _corners: &mut Corners) {
    if !game.native_on(m::ID) || game.player.is_none() {
        return;
    }
    let s = {
        let g = &*game;
        m::settings(|k| g.native_setting(m::ID, k))
    };
    if !s.enabled {
        return;
    }

    // Only render when in first person, not spectator, not in debug F3 crosshair
    if !game.camera_type.first_person() || game.spectator() || game.debug_crosshair() {
        return;
    }

    let (wi, hi) = game.gui_size();
    if wi <= 0.0 || hi <= 0.0 {
        return;
    }

    // Exact center pixel of vanilla crosshair (15x15 sprite centred at floor((w - 15)/2), floor((h - 15)/2))
    // In vanilla Horizon, sprite top-left is ((wi - 15) / 2).floor(), center pixel is at offset +7.0
    let cx = ((wi - 15.0) / 2.0).floor() + 7.0;
    let cy = ((hi - 15.0) / 2.0).floor() + 7.0;

    // Check attack strength / weapon cooldown
    let speed = game.main_hand_attack_speed();
    let strength = game.hand_strength.scale(0.0, speed);
    let charged = strength >= 0.98;

    // Check if aiming at a living entity in attack reach
    let target_in_reach = game.crosshair_living().is_some();

    // 1. Draw target indicator if hovering an entity in reach
    if target_in_reach {
        let color = s.theme.resolve(charged);

        match s.style {
            m::Style::Brackets => {
                // Exact PvP target brackets hugging the vanilla 9x9 crosshair inside corners:
                // Top-Left corner:
                game.native_fill(batch, [cx - 3.0, cy - 3.0, 2.0, 1.0], color);
                game.native_fill(batch, [cx - 3.0, cy - 2.0, 1.0, 1.0], color);
                // Top-Right corner:
                game.native_fill(batch, [cx + 2.0, cy - 3.0, 2.0, 1.0], color);
                game.native_fill(batch, [cx + 3.0, cy - 2.0, 1.0, 1.0], color);
                // Bottom-Left corner:
                game.native_fill(batch, [cx - 3.0, cy + 2.0, 1.0, 1.0], color);
                game.native_fill(batch, [cx - 3.0, cy + 3.0, 2.0, 1.0], color);
                // Bottom-Right corner:
                game.native_fill(batch, [cx + 3.0, cy + 2.0, 1.0, 1.0], color);
                game.native_fill(batch, [cx + 2.0, cy + 3.0, 2.0, 1.0], color);
            }
            m::Style::CenterDot => {
                // Bright 1x1 dot centered inside crosshair
                game.native_fill(batch, [cx, cy, 1.0, 1.0], color);
            }
            m::Style::Diamond => {
                // Diamond accent pips around the crosshair extremities
                game.native_fill(batch, [cx, cy - 5.0, 1.0, 1.0], color);
                game.native_fill(batch, [cx, cy + 5.0, 1.0, 1.0], color);
                game.native_fill(batch, [cx - 5.0, cy, 1.0, 1.0], color);
                game.native_fill(batch, [cx + 5.0, cy, 1.0, 1.0], color);
            }
            m::Style::Chevron => {
                // Tactical target chevron above crosshair
                game.native_fill(batch, [cx - 2.0, cy - 6.0, 5.0, 1.0], color);
                game.native_fill(batch, [cx - 1.0, cy - 5.0, 3.0, 1.0], color);
                game.native_fill(batch, [cx, cy - 4.0, 1.0, 1.0], color);
            }
        }
    }

    // 2. Optional mini cooldown bar below crosshair when weapon is charging
    if s.show_cooldown && !charged {
        let bar_w = 16.0;
        let bar_h = 2.0;
        let bar_x = (cx - 8.0).floor();
        let bar_y = cy + 10.0;
        // Background
        game.native_fill(batch, [bar_x, bar_y, bar_w, bar_h], m::COLOR_GRAY);
        // Progress fill
        let fill_w = (bar_w * strength.clamp(0.0, 1.0)).round();
        if fill_w > 0.0 {
            game.native_fill(batch, [bar_x, bar_y, fill_w, bar_h], 0xFFFFFFFF);
        }
    }
}

pub(super) fn on_key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !pressed || !game.native_on(m::ID) || !game.native_playing() {
        return;
    }
    let mut s = {
        let g = &*game;
        m::settings(|k| g.native_setting(m::ID, k))
    };

    let toggle_key = game.native_mod_key(m::ID, m::KEY_TOGGLE, m::KEY_TOGGLE_DEFAULT);
    if key == toggle_key && toggle_key != "key.keyboard.unknown" {
        s.enabled = !s.enabled;
        m::set_settings(s.clone());
        for (k, v) in s.stored() {
            game.native_set_setting(m::ID, k, &v);
        }
    }
}

pub(super) fn on_list(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: vec![[
            m::KEY_TOGGLE.into(),
            m::KEY_TOGGLE_LABEL.into(),
            m::KEY_TOGGLE_DEFAULT.into(),
        ]],
    });
}

pub(super) fn on_settings(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id != m::ID {
        return;
    }
    let g = &*game;
    let s = m::settings(|key| g.native_setting(m::ID, key));
    buttons.extend(s.labels());
}

pub(super) fn on_click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    if mod_id != m::ID {
        return;
    }
    let mut s = {
        let g = &*game;
        m::settings(|key| g.native_setting(m::ID, key))
    };
    s.click(button);
    m::set_settings(s.clone());
    for (key, value) in s.stored() {
        game.native_set_setting(m::ID, key, &value);
    }
}
