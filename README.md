# Verification notes for dan17612/pdfcraft draft branches

Draft branches in this fork were written without a Rust toolchain and are built and
tested by a Claude Code cloud session. It records one file per branch here
(`<branch-with-slashes-as-dashes>.md`): the commit verified, the commands with
pass/fail, what it changed, and evidence that each new test fails without the fix.

`patches/craft-libs-menu-keys.patch` is the craft-libs side of
`fix/menu-shortcut-key-names` (adds `MenuBarUi::key_names` to craft-menu and
`Chord::display_with` to craft-keymap). Base: storytold/craft-libs main 1a11a48
(craft-keymap and craft-menu are identical at e11f9d9, the rev pdfcraft pins).
