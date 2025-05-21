// Imports
use druid::{
    Data, Lens, Env,
    AppLauncher, WindowDesc, PlatformError,
    Widget, WidgetExt,
    Rect, Color, FontDescriptor, FontFamily, FontWeight,
    RenderContext, PaintCtx, EventCtx
};

use druid::widget::{
    Label, Flex, Painter
};

// ErrorType
type PrErr<T> = Result<T, String>;

// BoardDefinition
#[derive(Clone, Data, Lens)]
struct Board {
    answer: String,
    guesses: [[char; 5]; 6],
    correction: [[usize; 5]; 6],
    status: BoardStatus,

    
    gui_current_loc: [usize; 2], // for GUI only
    #[data(ignore)] gui_letter_hint: [usize; 26],
    gui_theme_no: usize
    
} impl Board {
    fn create() -> Board {
        Board {
            answer: Board::get_word(),
            guesses: [['·'; 5]; 6],
            correction: [[0; 5]; 6],
            status: BoardStatus::NotStarted,

            gui_current_loc: [0, 0],
            gui_letter_hint: [0; 26],
            gui_theme_no: 0
        }
    }

    fn get_word() -> String {
        use std::fs::File;
        use std::io::{BufReader, BufRead};
        use rand::prelude::{thread_rng, Rng};

        fn uhoh(e: String) -> String {
            println!("{}", e);
            String::from("HELLO")
        }

        let mut words = match File::open("src/dictionary.txt") {
            Ok(f)   => BufReader::new(f).lines(),
            Err(e)  => return uhoh(format!("{}", e))
        };

        let n: usize = match thread_rng().gen_range(0..5000).try_into() {
            Ok(n)   => n,
            Err(e)  => return uhoh(format!("{}", e))
        };

        match words.nth(n) {
            Some(line)   => match line {
                Ok(l)   => l.to_ascii_uppercase(),
                Err(e)  => return uhoh(format!("{}", e))
            },
            None      => return uhoh(String::from("No Word as nth()"))
        }
    }

    fn submit_guess(&mut self) -> PrErr<[usize; 5]> {
        let current = match self.status {
            BoardStatus::OnGoing(n) => { self.status = BoardStatus::OnGoing(n + 1);  n + 1  },
            BoardStatus::NotStarted => { self.status = BoardStatus::OnGoing(0);      0      },
            _                           => return Err(String::from("implBoard: Guess made, not NotStarted || OnGoing"))
        };

        let correction = self.make_correction(current);
        match correction {
            Ok(c)   => {
                // 3 is good, everthing else flag as 0
                if !c.into_iter().map( |x| match x % 5 { 3 => 1, _ => 0 } ).collect::<Vec<usize>>().contains(&0) {
                    self.status = BoardStatus::Win(current);
                } else if current == 5 { // all guesses made, and not win
                    self.status = BoardStatus::Lose;
                }
            },
            Err(e)  => return Err(e)
        };

        correction
    }

    fn make_correction(&mut self, current: usize) -> PrErr<[usize; 5]> {
        /*
            Working:
                (mod 5)
                0 if not calulated
                1 if not found
                2 if found in wrong place
                3 if found in right place
                4 if error

                (+ quotient 5)
                freq - 1

                eg. 17 = 4 occurances, current one is in right place
        */

        for (i, c) in self.guesses[current].into_iter().enumerate() {
            self.correction[current][i] = 1;
            if self.answer.contains(c) {
                self.correction[current][i] += 
                    1 + ((self.answer.chars().nth(i) == Some(c)) as usize) +
                    5 * (self.answer.matches(c).count() - 1);
            }

            let alphano = (c as usize) - 65;
            self.gui_letter_hint[alphano] = self.gui_letter_hint[alphano].max(self.correction[current][i] % 5);
        }

        Ok(self.correction[current])
    }
}

#[derive(Clone)]
enum BoardStatus {
    Win(usize),
    Lose,
    OnGoing(usize),
    NotStarted
} impl Data for BoardStatus {
    fn same(&self, other: &Self) -> bool {
        match self {
            BoardStatus::Win(x) => match other { BoardStatus::Win(y) => x == y, _ => false }
            BoardStatus::Lose => match other { BoardStatus::Lose => true, _ => false }
            BoardStatus::OnGoing(x) => match other { BoardStatus::OnGoing(y) => x == y, _ => false }
            BoardStatus::NotStarted => match other { BoardStatus::NotStarted => true, _ => false }
        }
    }
}

// Main
fn main() -> Result<(), PlatformError> {
    let game_window = WindowDesc::new(board_ui())
        .title("Wordle_Beta")
        .window_size((1500.0, 750.0));

    let game = Board::create();

    AppLauncher::with_window(game_window)
        .log_to_console()
        .launch(game)
}

// UI
const SIZE: f64 = 100.0 / 2.0;
const SPACE: f64 = 20.0 / 2.0;
const THEMATICS: [[Color; 6]; 4] = [
    /* 
        [
            N/A, 
            Wrong, 
            Right but diff place,
            Right,
            Selected Border,
            BG
        ]
    */

    [
        Color::rgb8(54, 52, 50),
	    Color::rgb8(239, 96, 36),
        Color::rgb8(240, 148, 31),
	    Color::rgb8(25, 103, 116),
        Color::grey8(0x50),
        Color::WHITE
    ],
    [
		Color::rgb8(0, 88, 91),
	    Color::rgb8(36, 80, 112),
		Color::rgb8(143, 223, 136),
		Color::rgb8(45, 166, 108),
		Color::rgb8(181, 232, 174),
		Color::rgb8(239, 247, 233)
    ],
    [
		Color::rgb8(18, 18, 19),
		Color::rgb8(58, 58, 60),
		Color::rgb8(181, 159, 59),
		Color::rgb8(83, 141, 78),
		Color::rgb8(129, 131, 132),
        Color::rgb8(18, 18, 19)
    ],
    [
		Color::rgb8(216, 180, 149),
		Color::rgb8(67, 64, 89),
		Color::rgb8(191, 128, 105),
		Color::rgb8(165, 104, 115),
        Color::rgb8(242, 226, 196),
		Color::rgb8(242, 226, 196)
    ]
];

fn board_ui() -> impl Widget<Board> {
    let mont: FontDescriptor = FontDescriptor::new(
            FontFamily::new_unchecked("Montserrat")
        )
        .with_weight(FontWeight::SEMI_BOLD)
        .with_size(20.0);

    let mut guess_ui: Flex<Board> = Flex::column()
        .with_flex_spacer(SPACE);

    let mut keyboard: Flex<Board> = Flex::column()
        .with_flex_spacer(SPACE);

    let mut theme_switch: Flex<Board> = Flex::column()
        .with_flex_spacer(SPACE);

    for i in 0..6 {
        let mut guess_ui_r: Flex<Board> = Flex::row();

        for j in 0..5 {
            guess_ui_r.add_child(
                Flex::column()
                    .with_flex_spacer(SPACE / 4.0)
                    .with_child(
                        Flex::row()
                            .with_flex_spacer(SPACE / 8.0)
                            .with_child(
                                Label::new(
                                    {
                                        let row = i; let col = j;
                                        move |data: &Board, _env: &Env| {
                                            String::from(data.guesses[row][col])
                                        }
                                    }
                                )
                                .with_font(mont.clone())
                            )
                            .with_child(
                                Label::new(
                                    {
                                        let row = i; let col = j;
                                        move |data: &Board, _env: &Env| {
                                            match String::from_utf16(&[
                                                match u16::try_from((data.correction[row][col] / 5) + 1 + 0x2080) { 
                                                    Ok(0x2081)  => 0x20,
                                                    Ok(n)       => n, 
                                                    Err(_)      => 0x2080
                                                }
                                            ]) { Ok(x) => x, Err(_) => String::from("₀") }
                                        }
                                    }
                                )
                                .with_font(mont.clone())
                            )
                            .with_flex_spacer(SPACE / 8.0)
                            .fix_width(SIZE)
                    )
                    .with_flex_spacer(SPACE / 4.0)
                    .fix_height(SIZE * 1.5)
                    .background(
                        Painter::new(
                            {
                                let row = i; let col = j;
                                move |ctx: &mut PaintCtx, data: &Board, _env: &Env| {
                                    let rnd = ctx.size().to_rounded_rect(SIZE / 4.0);
                                    ctx.fill( rnd, &THEMATICS[data.gui_theme_no][ data.correction[row][col] % 5 ]);

                                    match data.status {
                                        BoardStatus::NotStarted | BoardStatus::OnGoing(_) => {
                                            if data.gui_current_loc == [row, col] {
                                                let pointer = Rect::new(
                                                    10.0, 10.0, 
                                                    12.0, SIZE * 1.5 - 10.0
                                                )
                                                .to_rounded_rect(SIZE * 0.1);
                                                ctx.fill(pointer, &THEMATICS[data.gui_theme_no][4]);
                                            }
                                        },
                                        _ => ()
                                    };
                                }
                            }
                        )
                    )
                    .on_click(
                        {
                            let row = i; let col = j;
                            move |_ctx: &mut EventCtx, data: &mut Board, _env: &Env| {
                                if data.gui_current_loc[0] == row {
                                    data.gui_current_loc[1] = col;
                                }
                            }
                        }
                    )
            );
            guess_ui_r.add_spacer(SPACE);
        }

        guess_ui.add_spacer(SPACE);
        guess_ui.add_child(guess_ui_r);
    }

    let keyboard_layout = [
        ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'], 
        ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', '.'], 
        ['Z', 'X', 'C', 'V', 'B', 'N', 'M', '.', '.', '.']
    ];

    for i in 0..keyboard_layout.len() {
        let mut keyboard_r: Flex<Board> = Flex::row();

        for j in 0..keyboard_layout[i].len() {
            if !keyboard_layout[i][j].is_alphabetic() {
                break;
            }

            keyboard_r.add_child(
                Flex::column()
                    .with_flex_spacer(SPACE / 4.0)
                    .with_child(
                        Flex::row()
                            .with_flex_spacer(SPACE / 8.0)
                            .with_child(
                                Label::new(
                                    {
                                        let row = i; let col = j;
                                        move |_data: &Board, _env: &Env| {
                                            String::from(keyboard_layout[row][col])
                                        }
                                    }
                                )
                                .with_font(mont.clone())
                            )
                            .with_flex_spacer(SPACE / 8.0)
                            .fix_width(SIZE)
                    )
                    .with_flex_spacer(SPACE / 4.0)
                    .fix_height(SIZE * 1.5)
                    .background(
                        {
                            let row = i; let col = j;
                            Painter::new(
                                move |ctx: &mut PaintCtx, data: &Board, _env: &Env| {
                                    let rnd = ctx.size().to_rounded_rect(SIZE / 4.0);
                                    let c = data.gui_letter_hint[(keyboard_layout[row][col] as usize) - 65];
                                    ctx.fill(rnd, &THEMATICS[data.gui_theme_no][c]);
                                }
                            )
                        }
                    )
                    .on_click(
                        {
                            let row = i; let col = j;
                            move |_ctx: &mut EventCtx, data: &mut Board, _env: &Env| {
                                match data.status {
                                    BoardStatus::NotStarted | BoardStatus::OnGoing(_) => {
                                        if data.gui_current_loc[1] < 5 {
                                            data.guesses[data.gui_current_loc[0]][data.gui_current_loc[1]] = keyboard_layout[row][col];
                                            data.gui_current_loc[1] = match data.gui_current_loc[1] + 1 {
                                                x if x < 5  => x,
                                                _           => 5
                                            };
                                        }
                                    }
                                    _ => ()
                                };
                            }
                        }
                    )
            );

            keyboard_r.add_spacer(SPACE);
        }

        keyboard.add_spacer(SPACE);
        keyboard.add_child(keyboard_r);
    }

    let theme_len = THEMATICS.len();
    for i in 0..theme_len {
        theme_switch.add_child(
            Painter::new(
                {
                    let c = i;
                    move |ctx: &mut PaintCtx, _data: &Board, _env: &Env| {
                        let rnd = ctx.size().to_rounded_rect(SIZE / 2.0);
                        ctx.fill(rnd, &THEMATICS[c][3]);
                    }
                }
            )
            .fix_height(SIZE)
            .fix_width(SIZE)
            .on_click(
                {
                    let c = i;
                    move |_ctx: &mut EventCtx, data: &mut Board, _env: &Env| {
                        data.gui_theme_no = c;
                    }
                }
            )
        );
        theme_switch.add_spacer(SPACE);
    }

    keyboard.add_spacer(SPACE * 1.5);
    keyboard.add_child(
        Flex::row()
            .with_flex_spacer(SPACE / 4.0)
            .with_child(
                Flex::column()
                    .with_flex_spacer(SPACE / 8.0)
                    .with_child(
                        Label::new(String::from("SUBMIT"))
                            .with_font(mont.clone())
                    )
                    .with_flex_spacer(SPACE / 8.0)
                    .fix_height(SIZE * 1.5)
                    .fix_width(SIZE * 4.0)
                    .background(
                        Painter::new(
                            move |ctx: &mut PaintCtx, data: &Board, _env: &Env| {
                                let rnd = ctx.size().to_rounded_rect(SIZE / 4.0);
                                ctx.fill(rnd, &THEMATICS[data.gui_theme_no][0]);
                            }
                        )
                    )
                    .on_click(
                        move |_ctx: &mut EventCtx, data: &mut Board, _env: &Env| {
                            match data.status {
                                BoardStatus::NotStarted | BoardStatus::OnGoing(_) => {
                                    if !data.guesses[data.gui_current_loc[0]].contains(&'·') {
                                        match data.submit_guess() {
                                            Ok(_)  => (),
                                            Err(e)  => println!("{}", e)
                                        };

                                        data.gui_current_loc = [match data.gui_current_loc[0] + 1 {x if x < 5 => x, _ => 5}, 0];      
                                    }
                                }
                                _ => ()
                            };
                        }
                    )
            )
            .with_spacer(SPACE)
            .with_child(
                Flex::column()
                    .with_flex_spacer(SPACE / 8.0)
                    .with_child(
                        Label::new(String::from("<<"))
                            .with_font(mont.clone())
                    )
                    .with_flex_spacer(SPACE / 8.0)
                    .fix_height(SIZE * 1.5)
                    .fix_width(SIZE * 1.5)
                    .background(
                        Painter::new(
                            move |ctx: &mut PaintCtx, data: &Board, _env: &Env| {
                                let rnd = ctx.size().to_rounded_rect(SIZE / 4.0);
                                ctx.fill(rnd, &THEMATICS[data.gui_theme_no][0]);
                            }
                        )
                    )
                    .on_click(
                        move |_ctx: &mut EventCtx, data: &mut Board, _env: &Env| {
                            match data.status {
                                BoardStatus::NotStarted | BoardStatus::OnGoing(_) => {
                                    data.gui_current_loc[1] = match data.gui_current_loc[1].checked_sub(1) {
                                        Some(x) => x,
                                        None    => 0
                                    };
                                    data.guesses[data.gui_current_loc[0]][data.gui_current_loc[1]] = '·';
                                }
                                _ => ()
                            };
                        }
                    )
            )
            .with_flex_spacer(SPACE / 4.0)
            .fix_width(SIZE * 4.0 + SPACE + SIZE * 1.5)
    );
    keyboard.add_spacer(SPACE * 1.5);
    keyboard.add_child(
        Flex::row()
            .with_flex_spacer(SPACE / 4.0)
            .with_child(
                Flex::column()
                    .with_flex_spacer(SPACE / 8.0)
                    .with_child(
                        Label::new(
                            move |data: &Board, _env: &Env| {
                                match data.status {
                                    BoardStatus::Win(n) => format!("You have won in {} tries!", n + 1),
                                    BoardStatus::Lose   => format!("You have lost! The word was {}.", data.answer),
                                    _                   => String::new()
                                }
                            }
                        )
                        .with_font(mont.clone())
                    )
                    .with_flex_spacer(SPACE / 8.0)
                    .fix_height(SIZE * 1.5)
            )
            .with_flex_spacer(SPACE / 4.0)
            .fix_width(SIZE * 10.0 + SPACE * 9.0)
            .background(
                Painter::new(
                    move |ctx: &mut PaintCtx, data: &Board, _env: &Env| {
                        let rnd = ctx.size().to_rounded_rect(SIZE / 4.0);
                        let c = match data.status {
                            BoardStatus::Win(_) => &THEMATICS[data.gui_theme_no][3],
                            BoardStatus::Lose   => &THEMATICS[data.gui_theme_no][1],
                            _                   => &THEMATICS[data.gui_theme_no][0]
                        };

                        ctx.fill(rnd, c);
                    }
                )
            )
    );

    Flex::row()
        .with_flex_spacer(SPACE)
        .with_child(
            guess_ui
                .with_flex_spacer(SPACE)
                .fix_height(
                    SIZE * 1.5 * 6.0 + SPACE * 5.0 + SPACE * 2.0
                )
        )
        .with_spacer(SPACE * 4.0)
        .with_child(
            Flex::column()
                .fix_height(SIZE * 7.5)
                .fix_width(1.0)
                .background(Color::GRAY)
        )
        .with_spacer(SPACE * 5.0)
        .with_child(
            keyboard
                .with_flex_spacer(SPACE)
                .fix_height(
                    (SIZE * 1.5 * 3.0 + SPACE * 2.0) + (SPACE * 1.5 + SIZE * 1.5) + (SPACE * 1.5 + SIZE * 1.5) + SPACE * 2.0
                )
        )
        .with_spacer(SPACE * 4.0)
        .with_child(
            Flex::column()
                .fix_height(SIZE * 7.5)
                .fix_width(1.0)
                .background(Color::GRAY)
        )
        .with_spacer(SPACE * 5.0)
        .with_child(
            theme_switch
                .with_flex_spacer(SPACE)
                .fix_height(SIZE * (theme_len as f64) + SPACE * (theme_len as f64) + SPACE * 2.0)
        )
        .with_flex_spacer(SPACE)
        .fix_width(
            SPACE + (SIZE * 5.0 + SPACE * 5.0) + (SPACE * 9.0 + 1.0) + (SIZE * 9.0 + SPACE * 9.0) + (SPACE * 9.0 + 1.0) + (SIZE) + SPACE
        )
        .background(
            Painter::new(
                move |ctx: &mut PaintCtx, data: &Board, _env: &Env| {
                    let bg = ctx.size().to_rect();
                    ctx.fill(bg, &THEMATICS[data.gui_theme_no][5]);
                }
            )
        )
}