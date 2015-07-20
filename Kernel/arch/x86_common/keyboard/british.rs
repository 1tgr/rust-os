use super::Key;
use super::keys::*;

pub static KEYS: [Key; 94] = [
    //  Normal Shift  Ctrl   CtrlS  AltGr  AGrS
    Key(0x0000,0x0000,0x0000,0x0000,0x0000,0x0000),  // 0
    Key(27,    27,    27,    27,    0,     0),    // Esc
    Key('1' as u32,   '!' as u32,   '1' as u32,   '!' as u32,   0,     0),    // 1
    Key('2' as u32,   '"' as u32,   '2' as u32,   '"' as u32,   0,     0),    // 2
    Key('3' as u32,   0x00a3,'3' as u32,   0x00a3,0,     0),    // 3
    Key('4' as u32,   '$' as u32,   '4' as u32,   '$' as u32,   0x20ac,0),    // 4
    Key('5' as u32,   '%' as u32,   '5' as u32,   '%' as u32,   0,     0),    // 5
    Key('6' as u32,   '^' as u32,   '6' as u32,   '^' as u32,   0,     0),    // 6
    Key('7' as u32,   '&' as u32,   '7' as u32,   '&' as u32,   0,     0),    // 7
    Key('8' as u32,   '*' as u32,   '8' as u32,   '*' as u32,   0,     0),    // 8
    Key('9' as u32,   '(' as u32,   '9' as u32,   '(' as u32,   0,     0),    // 9
    Key('0' as u32,   ')' as u32,   '0' as u32,   ')' as u32,   0,     0),    // 0
    Key('-' as u32,   '_' as u32,   '-' as u32,   '_' as u32,   0,     0),    // -
    Key('=' as u32,   '+' as u32,   '+' as u32,   '+' as u32,   0,     0),    // =
    Key(8,     8,    8,      8,     0,     0),    // Backspace
    Key('\t' as u32,  '\t' as u32,  '\t' as u32,  '\t' as u32,  0,     0),    // Tab
    Key('q' as u32,   'Q' as u32,   'q' as u32,   'Q' as u32,   0,     0),    // Q
    Key('w' as u32,   'W' as u32,   'w' as u32,   'W' as u32,   0,     0),    // W
    Key('e' as u32,   'E' as u32,   'e' as u32,   'E' as u32,   0,     0),    // E
    Key('r' as u32,   'R' as u32,   'r' as u32,   'R' as u32,   0,     0),    // R
    Key('t' as u32,   'T' as u32,   't' as u32,   'T' as u32,   0,     0),    // T
    Key('y' as u32,   'Y' as u32,   'y' as u32,   'Y' as u32,   0,     0),    // Y
    Key('u' as u32,   'U' as u32,   'u' as u32,   'U' as u32,   0,     0),    // U
    Key('i' as u32,   'I' as u32,   'i' as u32,   'I' as u32,   0,     0),    // I
    Key('o' as u32,   'O' as u32,   'o' as u32,   'O' as u32,   0,     0),    // O
    Key('p' as u32,   'P' as u32,   'p' as u32,   'P' as u32,   0,     0),    // P
    Key('[' as u32,   '{' as u32,   ']' as u32,   '}' as u32,   0,     0),    // [
    Key(']' as u32,   '}' as u32,   ']' as u32,   '}' as u32,   0,     0),    // ]
    Key('\n' as u32,  '\n' as u32,  '\n' as u32,  '\n' as u32,  0,     0),    // Return
    Key(0,     0,     0,     0,     0,     0),    // Control
    Key('a' as u32,   'A' as u32,   'a' as u32,   'A' as u32,   0,     0),    // A
    Key('s' as u32,   'S' as u32,   's' as u32,   'S' as u32,   0,     0),    // S
    Key('d' as u32,   'D' as u32,   'd' as u32,   'D' as u32,   0,     0),    // D
    Key('f' as u32,   'F' as u32,   'f' as u32,   'F' as u32,   0,     0),    // F
    Key('g' as u32,   'G' as u32,   'g' as u32,   'G' as u32,   0,     0),    // G
    Key('h' as u32,   'H' as u32,   'h' as u32,   'H' as u32,   0,     0),    // H
    Key('j' as u32,   'J' as u32,   'j' as u32,   'J' as u32,   0,     0),    // J
    Key('k' as u32,   'K' as u32,   'k' as u32,   'K' as u32,   0,     0),    // K
    Key('l' as u32,   'L' as u32,   'l' as u32,   'L' as u32,   0,     0),    // L
    Key(';' as u32,   ':' as u32,   ';' as u32,   ':' as u32,   0,     0),    // ;
    Key('\'' as u32,  '@' as u32,   '\'' as u32,  '@' as u32,   0,     0),    // '
    Key('`' as u32,   '¬' as u32,   '`' as u32,   '¬' as u32,   '¦' as u32,   0),    // `
    Key(0,     0,     0,     0,     0,     0),    // Left Shift
    Key('#' as u32,   '~' as u32,   '#' as u32,   '~' as u32,   0,     0),    // #
    Key('z' as u32,   'Z' as u32,   'z' as u32,   'Z' as u32,   0,     0),    // Z
    Key('x' as u32,   'X' as u32,   'x' as u32,   'X' as u32,   0,     0),    // X
    Key('c' as u32,   'C' as u32,   'c' as u32,   'C' as u32,   0,     0),    // C
    Key('v' as u32,   'V' as u32,   'v' as u32,   'V' as u32,   0,     0),    // V
    Key('b' as u32,   'B' as u32,   'b' as u32,   'B' as u32,   0,     0),    // B
    Key('n' as u32,   'N' as u32,   'n' as u32,   'N' as u32,   0,     0),    // N
    Key('m' as u32,   'M' as u32,   'm' as u32,   'M' as u32,   0,     0),    // M
    Key(',' as u32,   '<' as u32,   ',' as u32,   '<' as u32,   0,     0),    // ,
    Key('.' as u32,   '>' as u32,   '.' as u32,   '>' as u32,   0,     0),    // .
    Key('/' as u32,   '?' as u32,   '/' as u32,   '?' as u32,   0,     0),    // /
    Key(0,     0,     0,     0,     0,     0),    // Right Shift
    Key(PRTSC, 0, 0, 0, 0, 0,           ),    // Print Screen
    Key(0,     0,     0,     0,     0,     0),    // Left Alt
    Key(' ' as u32,   ' ' as u32,   ' ' as u32,   ' ' as u32,   0,     0),    // Space
    Key(0,     0,     0,     0,     0,     0),    // Caps Lock
    Key(F1,F1,F1,F1,0,     0),    // F1
    Key(F2,F2,F2,F2,0,     0),    // F2
    Key(F3,F3,F3,F3,0,     0),    // F3
    Key(F4,F4,F4,F4,0,     0),    // F4
    Key(F5,F5,F5,F5,0,     0),    // F5
    Key(F6,F6,F6,F6,0,     0),    // F6
    Key(F7,F7,F7,F7,0,     0),    // F7
    Key(F8,F8,F8,F8,0,     0),    // F8
    Key(F9,F9,F9,F9,0,     0),    // F9
    Key(F10, F10, F10, F10, 0, 0), // F10
    Key(0,     0,     0,     0,     0,     0,   ), // Num Lock
    Key(0,     0,     0,     0,     0,     0,   ), // Scroll Lock
    Key(HOME,HOME,HOME,HOME,0, 0), // Home
    Key(UP,  UP,  UP,  UP,  0, 0), // Up
    Key(PGUP,PGUP,PGUP,PGUP,0, 0), // Page Up
    Key('-' as u32,     '-' as u32,     0x2013,  0x2013,  0, 0), // Num -
    Key(LEFT,LEFT,LEFT,LEFT,0, 0), // Left
    Key(0,       0,       0,       0,       0, 0), // Num 5
    Key(RIGHT,RIGHT,RIGHT,RIGHT,0, 0), // Right
    Key('+' as u32,     '+' as u32,     '+' as u32,     '+' as u32,     0, 0), // Num +
    Key(END, END, END, END, 0, 0), // End
    Key(DOWN,DOWN,DOWN,DOWN,0, 0), // Down
    Key(PGDN,PGDN,PGDN,PGDN,0, 0), // Page Down
    Key(INS, INS, INS, INS, 0, 0), // Insert
    Key(DEL, DEL, DEL, DEL, 0, 0), // Delete
    Key(SYSR,SYSR,SYSR,SYSR,0, 0), // Sys Req
    Key(0,       0,       0,       0,       0, 0), // Scancode 55
    Key('\\' as u32,    '|' as u32,    '\\' as u32,    '|' as u32,      0, 0), // Scancode 56
    Key(F11, F11, F11, F11, 0, 0), // F11
    Key(F12, F12, F12, F12, 0, 0), // F12
    Key(0,       0,       0,       0,       0, 0), // Scancode 59
    Key(0,       0,       0,       0,       0, 0), // Scancode 5A
    Key(LWIN,LWIN,LWIN,LWIN,0, 0), // Left Windows
    Key(RWIN,RWIN,RWIN,RWIN,0, 0), // Right Windows
    Key(MENU,MENU,MENU,MENU,0, 0), // Context Menu
];
