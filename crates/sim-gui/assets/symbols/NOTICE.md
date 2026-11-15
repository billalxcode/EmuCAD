# SVG Symbol Sources

The files in this directory are embedded into `sim-gui` at compile time.
All files are ASCII SVG.

## Class Base Symbols

Most class base symbols are copied without modification from
`chris-pikul/electronic-symbols`:

https://github.com/chris-pikul/electronic-symbols

That project is licensed under the MIT License, copyright (c) 2022
Chris Pikul. The license text is reproduced below.

The following files are original simple geometry drawn for EmuCAD:

- `vcc.svg`
- `thermistor.svg`
- `transmission_line.svg`
- `diode_array.svg`
- `igbt.svg`
- `scr.svg`
- `triac.svg`
- `diac.svg`
- `connector.svg`
- `ic.svg`
- `flip_flop.svg`
- `regulator.svg`
- `dcdc.svg`
- `sensor.svg`
- `display.svg`
- `rf.svg`
- `behavioral.svg`
- `board.svg`
- `motor.svg`

`mcu.svg` is a simplified DIP outline adapted from `GenericIC.svg` on
Wikimedia Commons. The original was uploaded by Thomas Lake (Tswsl1989)
and later made generic by Fredvanner. It is released into the public
domain.

https://commons.wikimedia.org/wiki/File:GenericIC.svg

## Per-Entry Variant Code

Every catalog entry gets a unique `SymbolKind::Catalog` value. The
renderer draws the class base symbol plus a 3 by 3 variant code in the
top-left of the component body. The code uses nine bits from the catalog
index, so all 500 entries have a distinct rendered symbol. The variant
code is original geometry and adds no third-party license obligation.

## MIT License for chris-pikul/electronic-symbols

Copyright (c) 2022 Chris Pikul

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
