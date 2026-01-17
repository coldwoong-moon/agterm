use agterm::terminal::screen::TerminalScreen;

#[test]
fn test_insert_characters() {
    let mut screen = TerminalScreen::new(10, 3);
    // Write "ABCDEFGHIJ" on first line
    screen.process(b"ABCDEFGHIJ");

    // Move cursor to position 3 (after "ABC")
    let buffer = &screen.get_all_lines()[0];
    assert_eq!(buffer[0].c, 'A');
    assert_eq!(buffer[1].c, 'B');
    assert_eq!(buffer[2].c, 'C');
    assert_eq!(buffer[3].c, 'D');

    // Insert 2 characters at position 3: ESC [ 3 ; 1 H ESC [ 2 @
    screen.process(b"\x1b[1;4H\x1b[2@");

    // Result should be "ABC  DEFGH" (D,E,F,G,H shifted right, I,J dropped)
    let buffer = &screen.get_all_lines()[0];
    assert_eq!(buffer[0].c, 'A');
    assert_eq!(buffer[1].c, 'B');
    assert_eq!(buffer[2].c, 'C');
    assert_eq!(buffer[3].c, ' '); // Inserted blank
    assert_eq!(buffer[4].c, ' '); // Inserted blank
    assert_eq!(buffer[5].c, 'D');
    assert_eq!(buffer[6].c, 'E');
    assert_eq!(buffer[7].c, 'F');
    assert_eq!(buffer[8].c, 'G');
    assert_eq!(buffer[9].c, 'H');
}

#[test]
fn test_delete_characters() {
    let mut screen = TerminalScreen::new(10, 3);
    // Write "ABCDEFGHIJ" on first line
    screen.process(b"ABCDEFGHIJ");

    // Move cursor to position 3 (at "D") and delete 2 characters: ESC [ 1 ; 4 H ESC [ 2 P
    screen.process(b"\x1b[1;4H\x1b[2P");

    // Result should be "ABCFGHIJ  " (D and E deleted, rest shifted left)
    let buffer = &screen.get_all_lines()[0];
    assert_eq!(buffer[0].c, 'A');
    assert_eq!(buffer[1].c, 'B');
    assert_eq!(buffer[2].c, 'C');
    assert_eq!(buffer[3].c, 'F');
    assert_eq!(buffer[4].c, 'G');
    assert_eq!(buffer[5].c, 'H');
    assert_eq!(buffer[6].c, 'I');
    assert_eq!(buffer[7].c, 'J');
    assert_eq!(buffer[8].c, ' '); // Blank fill
    assert_eq!(buffer[9].c, ' '); // Blank fill
}

#[test]
fn test_insert_characters_default_param() {
    let mut screen = TerminalScreen::new(10, 3);
    screen.process(b"ABCDEFGHIJ");

    // Insert 1 character (default when no param): ESC [ 1 ; 4 H ESC [ @
    screen.process(b"\x1b[1;4H\x1b[@");

    // Result should be "ABC DEFGHI"
    let buffer = &screen.get_all_lines()[0];
    assert_eq!(buffer[3].c, ' ');
    assert_eq!(buffer[4].c, 'D');
    assert_eq!(buffer[9].c, 'I');
}

#[test]
fn test_delete_characters_default_param() {
    let mut screen = TerminalScreen::new(10, 3);
    screen.process(b"ABCDEFGHIJ");

    // Delete 1 character (default when no param): ESC [ 1 ; 4 H ESC [ P
    screen.process(b"\x1b[1;4H\x1b[P");

    // Result should be "ABCEFGHIJ "
    let buffer = &screen.get_all_lines()[0];
    assert_eq!(buffer[2].c, 'C');
    assert_eq!(buffer[3].c, 'E');
    assert_eq!(buffer[4].c, 'F');
    assert_eq!(buffer[9].c, ' ');
}
