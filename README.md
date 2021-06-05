# chimper

[![Build Status](https://travis-ci.com/pedrocr/chimper.svg?branch=master)](https://travis-ci.com/pedrocr/chimper)
[![Crates.io](https://img.shields.io/crates/v/chimper.svg)](https://crates.io/crates/chimper)

![Screenshot](/images/screenshot.png?raw=true)

This is a graphical image viewer and editor that browses directories and supports all sorts of image formats. It is written in 100% rust code and depends only on rust code as well so should be safe to use for all kinds of weird formats out there.

Current State
-------------

All the basic browsing and viewing features should be working fine. There's also experimental support for editting files (click the chimp logo when an image is open).

Install
-------

If you have a recent rust toolchain just install the latest release with:

    cargo install chimper

This will get you a chimper binary that can be used wherever you want. The entire program, including all assets are included in the binary itself so you can just move that single file wherever you want.

Usage
-----

To use it just run it directly:

    # to start it browsing the current dir
    chimper
    
    # to start viewing a given file (JPG/PNG/RAW/etc)
    chimper some_file.foo
    
    # to start it browsing a specific dir
    chimper some/dir/somewhere

Keyboard Shortcuts
------------------

* <kbd>Esc</kbd> — Quit
* <kbd>F11</kbd> — Toggle fullscreen
* <kbd>Tab</kbd> — Toggle side bar

Contributing
------------

Bug reports and pull requests welcome at https://github.com/pedrocr/chimper

Meet us at #chimper on irc.libera.chat if you need to discuss a feature or issue in detail or even just for general chat. To just start chatting go to https://web.libera.chat/#chimper
