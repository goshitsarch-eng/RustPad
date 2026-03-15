# RustPad

RustPad is a small text editor built with Rust and Iced. It takes a lot of visual cues from old school Notepad, but it also has a few useful extras so it does not feel stuck in the past.

Right now it covers the basics well. You can open and save files, use find and replace, jump to a line, change fonts, toggle word wrap, switch on dark mode, and use a right click context menu for common edit actions.

If you want to run it locally, use `cargo run`.

Printing goes through your system `lp` setup, so that part only works if your machine already has a printer configured and the print service is running.
