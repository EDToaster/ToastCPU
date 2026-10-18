// VGA text-mode showcase: title, palette, the full 256-glyph font and a live
// counter. Handy for eyeballing the emulator's front-ends side by side:
//   make vga_demo.mif && cargo run --release -p emu -- [-w] vga_demo.mif
#include<std/io>
#include<std/math>

using io

inline GRID_X 4
inline GRID_Y 22
inline GRID_STYLE 0x3900 // white on blue

// move the print cursor to (x, y)
fn goto u16 u16 -> {
    over 100 swap - text_progress store
    100 math::mul + as(u16*) text_ptr store
}

fn palette -> {
    0 while { dup 8 < } {
        dup bg
        0 while { dup 8 < } {
            dup fg
            " Ab" print
            ++
        } drop
        ++
        // next row, indented
        BLACK bg 4 over 6 + goto
    } drop
}

// write all 256 glyphs straight into VRAM (bypassing print_char, which
// treats 0x0A as a newline)
fn glyphs -> {
    0 while { dup 256 < } {
        dup 4 >> GRID_Y + 100 math::mul
        over 0xF & 1 << GRID_X + +
        over GRID_STYLE | swap as(u16*) store
        ++
    } drop
}

fn main -> {
    clear

    YELLOW fg BLUE bg
    2 1 goto " ToastCPU VGA text mode - 100x60 cells, 6x8 font, 8 colours " print
    WHITE fg BLACK bg
    2 3 goto "The quick brown fox jumps over the lazy dog. 0123456789 !?#$%&*()[]{}<>" print

    4 5 goto "palette (fg across, bg down):" print
    4 6 goto palette

    4 20 goto "all 256 glyphs (codes 0x00-0xFF):" print
    glyphs

    GREEN fg
    4 40 goto "frames:" print
    0 while { 1 } {
        12 40 goto dup print_decimal
        ++
    } drop
}
