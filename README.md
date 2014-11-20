# rust-tinyasm

A Rust port of my Python submission for http://redd.it/1kqxz9:

> Tiny, a very simple fictional computer architecture, is programmed by an assembly language that has 16 mnemonics, with 37 unique op-codes. The system is based on Harvard architecture, and is very straight-forward: program memory is different from working memory, the machine only executes one instruction at a time, memory is an array of bytes from index 0 to index 255 (inclusive), and doesn't have any relative addressing modes.
>
Your goal will be to write an assembler for Tiny: though you don't need to simulate the code or machine components, you must take given assembly-language source code and produce a list of hex op-codes. You are essentially writing code that converts the lowest human-readable language to machine-readable language!

My original Python submission can be found here:
https://github.com/msiemens/TINY.ASM/. This is a Rust port. It features
a much better architecture, including a proper parser and abstract syntax tree.
Like the Python version, this also comes with a small VM.

## Usage

Run the assembler:

    $ tiny asm <input>

Create a binary file that the VM can execute:

    $ tiny asm --bin <input> <binary>

Run the VM:

    $ tiny vm <binary>


## Syntax (+ Additions)

     v--- operation
    MOV [0] 1
         ^  ^---- literal
         |------- memory address


**Comments**

    ; This is a comment

**Labels**

    label:
    JMP :label

**Constants**

    $mem_addr = [0]
    $some_const = 5

    MOV $mem_addr $some_const

**Imports**

    #import file_name.asm

**Char Constants**

    APRINT '!'  ; Prints: !
    APRINT '\n' ; Prints a newline

**Subroutines**

    ; Define a subroutine
    ; name ----v              v---- number of arguments
    @start(binary_shift_left, 1)
        ADD     $arg0       $arg0
        MOV     $return     $arg0
    @end

    ; Call a subroutine
    @call(binary_shift_left, 5)
    @call(binary_shift_left, [5])


## LICENSE

The MIT License (MIT)

Copyright (c) 2014 Markus Siemens

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.