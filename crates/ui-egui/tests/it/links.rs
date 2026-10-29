//! Community links: the Discord button is one click away everywhere; Help menu, About dialog and
//! home screen open the ArtCraft and PdfCraft pages.

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_engine::links;
use pdfcraft_ui_egui::{Dialog, PdfCraftApp};

/// A one-page PDF, for tests that need a document open.
const ONE_PAGE: &[u8] = b"%PDF-1.7\n1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj\n3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] >> endobj\ntrailer << /Root 1 0 R >>\n%%EOF";

/// The Preferences checkbox for the Discord button and the community card.
const SHOW_DISCORD: &str = "Show the Discord button and the community card";

fn harness(setup: impl FnOnce(&mut PdfCraftApp) + 'static) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        setup(&mut app);
        app
    });
    h.run_steps(4);
    h
}

#[test]
fn discord_button_in_the_top_bar_opens_discord() {
    let mut h = harness(|_| {});
    h.get_by_label("Discord").click();
    h.run_steps(2);
    assert_eq!(h.state().last_opened_url.as_deref(), Some(links::DISCORD));
    assert_eq!(links::DISCORD, "https://discord.gg/artcraft");
}

#[test]
fn home_screen_links() {
    for (label, url) in [
        ("Join our Discord", links::DISCORD),
        ("PdfCraft web page", "https://getartcraft.com/apps/pdfcraft"),
        ("PdfCraft on GitHub", "https://github.com/storytold/pdfcraft"),
        ("ArtCraft website", "https://getartcraft.com"),
    ] {
        let mut h = harness(|_| {});
        h.get_by_label("Join the ArtCraft community");
        h.get_by_label(label).click();
        h.run_steps(2);
        assert_eq!(h.state().last_opened_url.as_deref(), Some(url), "{label}");
    }
}

#[test]
fn home_community_card_closes_for_this_run() {
    let mut h = harness(|_| {});
    h.get_by_label("Join the ArtCraft community");
    h.get_by_label("Close").click();
    h.run_steps(2);
    assert!(h.state().community_card_closed);
    assert!(h.query_by_label("Join the ArtCraft community").is_none());
    assert!(h.query_by_label("Join our Discord").is_none());
    // Not remembered: a fresh app shows the card again.
    assert!(!PdfCraftApp::new().community_card_closed);
}

/// Preferences (or `--discord false`) turns the title bar's Discord button and the community
/// card off for good: both stay away after a restart, and Help ▸ Join the ArtCraft Discord
/// still opens Discord. Settings from before the preference show both.
#[test]
fn the_discord_button_and_community_card_can_be_turned_off_for_good() {
    let open = |settings: Option<String>| {
        harness(move |app| {
            if let Some(s) = &settings {
                app.restore(s);
            }
        })
    };
    let mut h = open(None);
    assert!(h.state().show_discord, "on by default");
    h.get_by_label("Discord");
    h.get_by_label("Join the ArtCraft community");
    h.state_mut().set_option("dialog", "preferences").unwrap();
    h.run_steps(2);
    h.get_by_role_and_label(Role::CheckBox, SHOW_DISCORD).scroll_to_me();
    h.run_steps(4);
    h.get_by_role_and_label(Role::CheckBox, SHOW_DISCORD).click();
    h.run_steps(2);
    assert!(!h.state().show_discord);
    h.state_mut().dialog = None;
    h.run_steps(2);
    assert!(h.query_by_label("Discord").is_none(), "no Discord button in the title bar");
    assert!(h.query_by_label("Join the ArtCraft community").is_none(), "no community card on Home");
    assert!(h.query_by_label("Join our Discord").is_none());
    // The Help menu keeps the link.
    assert!(h.state_mut().execute("help.discord"));
    assert_eq!(h.state().last_opened_url.as_deref(), Some(links::DISCORD));
    // Kept in the settings: the next launch shows neither.
    let saved = h.state().persist();
    let mut h = open(Some(saved));
    assert!(!h.state().show_discord);
    assert!(h.query_by_label("Discord").is_none(), "remembered across a restart");
    assert!(h.query_by_label("Join the ArtCraft community").is_none());
    // Back on from Preferences.
    h.state_mut().set_option("dialog", "preferences").unwrap();
    h.run_steps(2);
    h.get_by_role_and_label(Role::CheckBox, SHOW_DISCORD).scroll_to_me();
    h.run_steps(4);
    h.get_by_role_and_label(Role::CheckBox, SHOW_DISCORD).click();
    h.run_steps(2);
    h.state_mut().dialog = None;
    h.run_steps(2);
    h.get_by_label("Discord");
    h.get_by_label("Join the ArtCraft community");
    // Settings saved before the preference existed, or with an unreadable value, keep both.
    for old in ["{}", r#"{"show_discord": "no"}"#] {
        let mut app = PdfCraftApp::new();
        app.restore(old);
        assert!(app.show_discord, "{old}");
    }
    // Scriptable (`--discord`, `ui.set`), with a clear refusal for anything else.
    let app = h.state_mut();
    app.set_option("discord", "false").unwrap();
    assert!(!app.show_discord);
    app.set_option("discord", "true").unwrap();
    assert!(app.show_discord);
    assert_eq!(app.set_option("discord", "maybe"), Err("discord must be true or false".into()));
}

/// Without the Discord button the document tabs take its room, and Open still stays in sight
/// left of the icon buttons.
#[test]
fn without_the_discord_button_the_tabs_take_its_room() {
    let open_right = |show_discord: bool| {
        let h = harness(move |app| {
            app.show_discord = show_discord;
            for i in 0..20 {
                app.open_bytes(&format!("document-{i:02}.pdf"), None, ONE_PAGE.to_vec()).unwrap();
            }
        });
        assert_eq!(h.query_by_label("Discord").is_some(), show_discord);
        let open = h.get_by_label("Open").rect();
        let help = h.get_by_label("Keyboard shortcuts").rect();
        assert!(open.left() > 0.0 && open.right() <= help.left(), "Open {open:?} stays in sight left of {help:?}");
        open.right()
    };
    let with_button = open_right(true);
    let without = open_right(false);
    assert!(without > with_button + 40.0, "Open moves into the button's room: {with_button} to {without}");
}

#[test]
fn about_dialog_shows_the_brand_and_links() {
    // With a document open, so the home screen's own links are not on screen.
    let mut h = harness(move |app| {
        app.open_bytes("one.pdf", None, ONE_PAGE.to_vec()).unwrap();
        app.dialog = Some(Dialog::About);
    });
    assert!(h.query_all_by_label("ArtCraft").count() >= 2, "the mark and the wordmark (alt text)");
    h.get_by_label("Join our Discord").click();
    h.run_steps(2);
    assert_eq!(h.state().last_opened_url.as_deref(), Some(links::DISCORD));
}

#[test]
fn help_commands_open_each_link() {
    for l in links::LINKS {
        let mut h = harness(|_| {});
        assert!(h.state_mut().execute(l.command), "{}", l.command);
        assert_eq!(h.state().last_opened_url.as_deref(), Some(l.url));
        assert_eq!(pdfcraft_engine::commands::command(l.command).unwrap().menu, Some("Help"));
    }
}

/// Runs frames until the node labelled `label` exists and has stopped moving, then returns it.
///
/// The About dialog is a modal centred on its own size, and switching tabs changes that size, so
/// for a few frames afterwards the modal grows and recentres and every control in it moves. A
/// fixed number of frames is a guess at how long that takes: it was too few after the 0.5.0
/// contributors refresh (78 names instead of 5) on some machines, the click on Table landed where
/// the button had been, and the table never opened (#671).
fn settled<'a>(h: &'a mut Harness<'static, PdfCraftApp>, label: &'a str) -> egui_kittest::Node<'a> {
    let mut last = None;
    for _ in 0..60 {
        h.run_steps(1);
        let now = h.query_by_label(label).map(|n| n.rect());
        if now.is_some() && now == last {
            return h.get_by_label(label);
        }
        last = now;
    }
    panic!("{label:?} never appeared or never stopped moving");
}

#[test]
fn about_dialog_has_contributors_and_models_tabs() {
    let mut h = harness(|app| app.dialog = Some(Dialog::About));
    settled(&mut h, "Contributors").click();
    // The owner is always in the compiled-in credits (contributors/contributors.json), shown by username.
    settled(&mut h, "@echelon");
    settled(&mut h, "Table").click();
    settled(&mut h, "PRs");
    settled(&mut h, "Display name").click();
    settled(&mut h, "Brandon Thomas");
    settled(&mut h, "Models").click();
    h.run_steps(4);
    assert!(h.query_all_by_label("Anthropic").count() >= 1);
}
