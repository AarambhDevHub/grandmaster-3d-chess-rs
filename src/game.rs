use std::str::FromStr;

use chess::{Board, BoardStatus, ChessMove, Color, File, MoveGen, Piece, Rank, Square};

use crate::bot::{choose_bot_move, evaluate_board};
use crate::constants::{SAMPLE_PUZZLES, STORY_CHAPTERS};
use crate::types::{
    BossState, CameraMode, ChessPieceView, CinematicMode, Difficulty, GameMode, LastAction,
    MoveData, PieceKind, PlayerColor, Puzzle,
};

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Clone, Debug, Default)]
pub struct CapturedPieces {
    pub white: Vec<PieceKind>,
    pub black: Vec<PieceKind>,
}

#[derive(Clone, Debug)]
pub struct GameController {
    pub board: Board,
    pub fen: String,
    pub turn: PlayerColor,
    pub is_check: bool,
    pub is_game_over: bool,
    pub winner: Option<PlayerColor>,
    pub is_draw: bool,
    pub history: Vec<String>,
    pub history_uci: Vec<String>,
    pub history_fen: Vec<String>,
    pub captured: CapturedPieces,
    pub evaluation: i32,
    pub selected_square: Option<String>,
    pub valid_moves: Vec<String>,
    pub last_move: Option<MoveData>,
    pub last_action: Option<LastAction>,
    pub cinematic_mode: CinematicMode,
    pub commentary: String,
    pub game_mode: GameMode,
    pub difficulty: Difficulty,
    pub player_color: PlayerColor,
    pub camera_mode: CameraMode,
    pub promotion_pending: Option<MoveData>,
    pub is_replay_mode: bool,
    pub replay_index: usize,
    pub boss_state: BossState,
    pub puzzle: Option<Puzzle>,
    pub puzzle_move_index: usize,
    pub is_puzzle_solved: bool,
    pub hint_square: Option<String>,
}

impl Default for GameController {
    fn default() -> Self {
        let board = Board::default();
        Self {
            board,
            fen: START_FEN.to_string(),
            turn: PlayerColor::White,
            is_check: false,
            is_game_over: false,
            winner: None,
            is_draw: false,
            history: Vec::new(),
            history_uci: Vec::new(),
            history_fen: vec![START_FEN.to_string()],
            captured: CapturedPieces::default(),
            evaluation: 0,
            selected_square: None,
            valid_moves: Vec::new(),
            last_move: None,
            last_action: None,
            cinematic_mode: CinematicMode::None,
            commentary: "Welcome to Royal 3D Chess.".to_string(),
            game_mode: GameMode::PlayerVsPlayer,
            difficulty: Difficulty::Medium,
            player_color: PlayerColor::White,
            camera_mode: CameraMode::Orbit,
            promotion_pending: None,
            is_replay_mode: false,
            replay_index: 0,
            boss_state: BossState::default(),
            puzzle: None,
            puzzle_move_index: 0,
            is_puzzle_solved: false,
            hint_square: None,
        }
    }
}

impl GameController {
    pub fn init(&mut self, mode: GameMode, difficulty: Difficulty, player_name: &str) {
        *self = Self::default();
        self.game_mode = mode;
        self.difficulty = difficulty;
        self.commentary = format!("Match started. Good luck, {player_name}.");

        if mode == GameMode::Boss {
            self.boss_state = BossState {
                is_active: true,
                boss_hp: 2000,
                max_hp: 2000,
                player_hp: 100,
            };
            self.difficulty = Difficulty::Hard;
        }

        if mode == GameMode::Puzzles {
            self.load_puzzle(SAMPLE_PUZZLES[random_puzzle_index()]);
        }
    }

    pub fn start_story_chapter(&mut self, chapter_id: u32, player_name: &str) {
        let chapter = STORY_CHAPTERS
            .iter()
            .find(|chapter| chapter.id == chapter_id)
            .unwrap_or(&STORY_CHAPTERS[0]);
        self.init(GameMode::Story, chapter.difficulty, player_name);
        self.commentary = chapter.intro_text.to_string();
    }

    pub fn load_puzzle(&mut self, puzzle: Puzzle) {
        let board = Board::from_str(puzzle.fen).unwrap_or_default();
        self.board = board;
        self.fen = board.to_string();
        self.turn = color_from_chess(board.side_to_move());
        self.history.clear();
        self.history_uci.clear();
        self.history_fen = vec![self.fen.clone()];
        self.puzzle = Some(puzzle);
        self.puzzle_move_index = 0;
        self.is_puzzle_solved = false;
        self.hint_square = None;
        self.player_color = self.turn;
        self.commentary = format!("Puzzle: {} (Rating: {})", puzzle.description, puzzle.rating);
    }

    pub fn visible_fen(&self) -> &str {
        if self.is_replay_mode {
            self.history_fen
                .get(self.replay_index)
                .map(String::as_str)
                .unwrap_or(&self.fen)
        } else {
            &self.fen
        }
    }

    pub fn pieces(&self) -> Vec<ChessPieceView> {
        pieces_from_fen(self.visible_fen())
    }

    pub fn select_square(&mut self, square: &str) {
        if self.is_game_over || self.is_replay_mode || self.is_puzzle_solved {
            return;
        }
        if self.is_engine_turn() {
            return;
        }
        if self.selected_square.as_deref() == Some(square) {
            self.selected_square = None;
            self.valid_moves.clear();
            return;
        }
        let Some(sq) = square_from_name(square) else {
            return;
        };
        if self.board.color_on(sq) != Some(chess_color_from_player(self.turn)) {
            self.selected_square = None;
            self.valid_moves.clear();
            return;
        }
        self.selected_square = Some(square.to_string());
        self.valid_moves = MoveGen::new_legal(&self.board)
            .filter(|mv| mv.get_source() == sq)
            .map(|mv| square_name(mv.get_dest()))
            .collect();
    }

    pub fn click_square(&mut self, square: &str) -> bool {
        if let Some(from) = self.selected_square.clone() {
            if self.valid_moves.iter().any(|valid| valid == square) {
                return self.make_move(&from, square, None);
            }
        }
        self.select_square(square);
        false
    }

    pub fn make_move(&mut self, from: &str, to: &str, promotion: Option<PieceKind>) -> bool {
        if self.is_game_over || self.is_replay_mode {
            return false;
        }
        if self.game_mode == GameMode::Puzzles && !self.is_puzzle_solved {
            if let Some(puzzle) = self.puzzle {
                let expected = puzzle.moves.get(self.puzzle_move_index).copied();
                let attempted = format!(
                    "{from}{to}{}",
                    promotion
                        .map(|piece| promotion_char(piece).to_string())
                        .unwrap_or_default()
                );
                if expected.is_some() && expected != Some(attempted.as_str()) {
                    self.commentary = "Incorrect move. Try again.".to_string();
                    self.selected_square = None;
                    self.valid_moves.clear();
                    return false;
                }
            }
        }

        let moved = self.apply_move(from, to, promotion, true);
        if !moved {
            return false;
        }

        if self.game_mode == GameMode::Puzzles && !self.is_puzzle_solved {
            self.advance_puzzle();
        }

        true
    }

    pub fn set_promotion(&mut self, promotion: PieceKind) -> bool {
        let Some(pending) = self.promotion_pending.clone() else {
            return false;
        };
        self.promotion_pending = None;
        self.make_move(&pending.from, &pending.to, Some(promotion))
    }

    pub fn make_bot_move(&mut self) -> bool {
        if self.is_game_over {
            return false;
        }
        let Some(mv) = choose_bot_move(&self.board, self.difficulty) else {
            return false;
        };
        let from = square_name(mv.get_source());
        let to = square_name(mv.get_dest());
        let promotion = mv.get_promotion().map(piece_from_chess);
        self.apply_move(&from, &to, promotion, false)
    }

    pub fn undo(&mut self) {
        let remove_count = if matches!(
            self.game_mode,
            GameMode::PlayerVsAi | GameMode::Story | GameMode::Boss
        ) {
            2
        } else {
            1
        };
        for _ in 0..remove_count {
            if self.history_fen.len() <= 1 || self.history_uci.is_empty() {
                break;
            }
            self.history_fen.pop();
            self.history.pop();
            self.history_uci.pop();
        }
        let fen = self
            .history_fen
            .last()
            .cloned()
            .unwrap_or_else(|| START_FEN.to_string());
        self.board = Board::from_str(&fen).unwrap_or_default();
        self.sync_status();
        self.last_move = None;
        self.last_action = None;
        self.cinematic_mode = CinematicMode::None;
        self.recalculate_captured();
        self.selected_square = None;
        self.valid_moves.clear();
    }

    pub fn start_replay(&mut self) {
        self.is_replay_mode = true;
        self.replay_index = self.history_fen.len().saturating_sub(1);
        self.selected_square = None;
        self.valid_moves.clear();
    }

    pub fn stop_replay(&mut self) {
        self.is_replay_mode = false;
        self.replay_index = self.history_fen.len().saturating_sub(1);
    }

    pub fn set_replay_index(&mut self, index: usize) {
        if index < self.history_fen.len() {
            self.replay_index = index;
        }
    }

    pub fn next_replay_move(&mut self) {
        self.set_replay_index(
            (self.replay_index + 1).min(self.history_fen.len().saturating_sub(1)),
        );
    }

    pub fn prev_replay_move(&mut self) {
        self.set_replay_index(self.replay_index.saturating_sub(1));
    }

    pub fn show_puzzle_hint(&mut self) {
        if let Some(puzzle) = self.puzzle {
            if let Some(expected) = puzzle.moves.get(self.puzzle_move_index) {
                self.hint_square = Some(expected[0..2].to_string());
                self.commentary = "Hint shown.".to_string();
            }
        }
    }

    pub fn load_replay_from_uci(&mut self, moves: &str) {
        self.init(GameMode::PlayerVsPlayer, Difficulty::Medium, "Player");
        for token in moves.split_whitespace() {
            if token.len() >= 4 {
                let from = &token[0..2];
                let to = &token[2..4];
                let promotion = token
                    .as_bytes()
                    .get(4)
                    .and_then(|c| piece_from_promotion_char(*c as char));
                let _ = self.apply_move(from, to, promotion, true);
            }
        }
        self.start_replay();
    }

    fn apply_move(
        &mut self,
        from: &str,
        to: &str,
        promotion: Option<PieceKind>,
        allow_pending_promotion: bool,
    ) -> bool {
        let Some(from_square) = square_from_name(from) else {
            return false;
        };
        let Some(to_square) = square_from_name(to) else {
            return false;
        };
        let Some(moved_piece) = self.board.piece_on(from_square) else {
            return false;
        };
        let moved_color = self.board.color_on(from_square).unwrap_or(Color::White);
        let promotion_piece = promotion.map(piece_to_chess);
        let candidate = ChessMove::new(from_square, to_square, promotion_piece);

        if needs_promotion(self.board, from_square, to_square, moved_piece) && promotion.is_none() {
            if allow_pending_promotion {
                self.promotion_pending = Some(MoveData {
                    from: from.to_string(),
                    to: to.to_string(),
                    promotion: None,
                });
                self.commentary = "Promotion available.".to_string();
            }
            return false;
        }

        if !MoveGen::new_legal(&self.board).any(|legal| legal == candidate) {
            self.commentary = "Invalid move.".to_string();
            return false;
        }

        let captured_square = captured_square_for(&self.board, from_square, to_square, moved_piece);
        let captured_piece = captured_square.and_then(|sq| self.board.piece_on(sq));
        let before_turn = color_from_chess(moved_color);
        self.board = self.board.make_move_new(candidate);

        let uci = format!(
            "{from}{to}{}",
            promotion
                .map(|piece| promotion_char(piece).to_string())
                .unwrap_or_default()
        );
        let notation = notation_for(
            moved_piece,
            before_turn,
            from,
            to,
            captured_piece,
            promotion,
        );
        self.history.push(notation.clone());
        self.history_uci.push(uci);
        self.history_fen.push(self.board.to_string());
        self.last_move = Some(MoveData {
            from: from.to_string(),
            to: to.to_string(),
            promotion,
        });

        if let Some(piece) = captured_piece {
            let captured_kind = piece_from_chess(piece);
            if before_turn == PlayerColor::White {
                self.captured.white.push(captured_kind);
            } else {
                self.captured.black.push(captured_kind);
            }
            if self.boss_state.is_active {
                self.boss_state.boss_hp = (self.boss_state.boss_hp - 150).max(0);
            }
        }

        self.sync_status();
        self.evaluation = evaluate_board(&self.board);
        self.selected_square = None;
        self.valid_moves.clear();
        self.hint_square = None;
        self.promotion_pending = None;
        self.cinematic_mode = if self.is_game_over && self.winner.is_some() {
            CinematicMode::GiantKingEnding
        } else if captured_piece.is_some() {
            CinematicMode::KillCam
        } else {
            CinematicMode::None
        };
        self.last_action = Some(LastAction {
            action_type: if captured_piece.is_some() {
                "capture".to_string()
            } else {
                "move".to_string()
            },
            from: from.to_string(),
            to: to.to_string(),
            piece: piece_from_chess(moved_piece),
            captured_piece: captured_piece.map(piece_from_chess),
            captured_square: captured_square.map(square_name),
            is_check: self.is_check,
            is_checkmate: self.is_game_over && self.winner.is_some(),
        });

        self.commentary = if self.is_game_over {
            if let Some(winner) = self.winner {
                format!("CHECKMATE. {} wins.", winner.label())
            } else {
                "Draw.".to_string()
            }
        } else if self.is_check {
            "CHECK. The king is in danger.".to_string()
        } else if captured_piece.is_some() {
            format!("{} captures on {to}.", before_turn.label())
        } else {
            format!("{notation} - {} moves.", before_turn.label())
        };

        true
    }

    fn advance_puzzle(&mut self) {
        let Some(puzzle) = self.puzzle else {
            return;
        };
        self.puzzle_move_index += 1;
        if self.puzzle_move_index >= puzzle.moves.len() {
            self.is_puzzle_solved = true;
            self.commentary = "Puzzle solved. Excellent work.".to_string();
            return;
        }
        if self.puzzle_move_index % 2 == 1 {
            if let Some(response) = puzzle.moves.get(self.puzzle_move_index) {
                let promotion = response
                    .as_bytes()
                    .get(4)
                    .and_then(|c| piece_from_promotion_char(*c as char));
                let from = &response[0..2];
                let to = &response[2..4];
                if self.apply_move(from, to, promotion, false) {
                    self.puzzle_move_index += 1;
                    if self.puzzle_move_index >= puzzle.moves.len() {
                        self.is_puzzle_solved = true;
                        self.commentary = "Puzzle solved. Excellent work.".to_string();
                    }
                }
            }
        }
    }

    pub fn should_auto_bot_move(&self) -> bool {
        matches!(
            self.game_mode,
            GameMode::PlayerVsAi | GameMode::Story | GameMode::Boss
        ) && !self.is_game_over
            && self.turn != self.player_color
    }

    fn is_engine_turn(&self) -> bool {
        matches!(
            self.game_mode,
            GameMode::PlayerVsAi | GameMode::Story | GameMode::Boss | GameMode::Puzzles
        ) && self.turn != self.player_color
    }

    fn sync_status(&mut self) {
        self.fen = self.board.to_string();
        self.turn = color_from_chess(self.board.side_to_move());
        self.is_check = self.board.checkers().popcnt() > 0;
        match self.board.status() {
            BoardStatus::Ongoing => {
                self.is_game_over = false;
                self.is_draw = false;
                self.winner = None;
            }
            BoardStatus::Stalemate => {
                self.is_game_over = true;
                self.is_draw = true;
                self.winner = None;
            }
            BoardStatus::Checkmate => {
                self.is_game_over = true;
                self.is_draw = false;
                self.winner = Some(self.turn.opposite());
            }
        }
    }

    fn recalculate_captured(&mut self) {
        let history = self.history_uci.clone();
        self.captured = CapturedPieces::default();
        let mut board = Board::default();
        for uci in history {
            let from = &uci[0..2];
            let to = &uci[2..4];
            let Some(from_sq) = square_from_name(from) else {
                continue;
            };
            let Some(to_sq) = square_from_name(to) else {
                continue;
            };
            let Some(piece) = board.piece_on(from_sq) else {
                continue;
            };
            let Some(color) = board.color_on(from_sq) else {
                continue;
            };
            let capture = captured_square_for(&board, from_sq, to_sq, piece)
                .and_then(|sq| board.piece_on(sq));
            if let Some(captured) = capture {
                if color == Color::White {
                    self.captured.white.push(piece_from_chess(captured));
                } else {
                    self.captured.black.push(piece_from_chess(captured));
                }
            }
            let promotion = uci
                .as_bytes()
                .get(4)
                .and_then(|c| piece_from_promotion_char(*c as char));
            let mv = ChessMove::new(from_sq, to_sq, promotion.map(piece_to_chess));
            if MoveGen::new_legal(&board).any(|legal| legal == mv) {
                board = board.make_move_new(mv);
            }
        }
    }
}

pub fn square_from_name(name: &str) -> Option<Square> {
    let bytes = name.as_bytes();
    if bytes.len() != 2 {
        return None;
    }
    let file = match bytes[0] {
        b'a'..=b'h' => (bytes[0] - b'a') as usize,
        b'A'..=b'H' => (bytes[0] - b'A') as usize,
        _ => return None,
    };
    let rank = match bytes[1] {
        b'1'..=b'8' => (bytes[1] - b'1') as usize,
        _ => return None,
    };
    Some(Square::make_square(
        Rank::from_index(rank),
        File::from_index(file),
    ))
}

pub fn square_name(square: Square) -> String {
    let file = (b'a' + square.get_file().to_index() as u8) as char;
    let rank = (b'1' + square.get_rank().to_index() as u8) as char;
    format!("{file}{rank}")
}

pub fn pieces_from_fen(fen: &str) -> Vec<ChessPieceView> {
    let mut pieces = Vec::new();
    let board_part = fen.split_whitespace().next().unwrap_or_default();
    for (row_index, row) in board_part.split('/').enumerate() {
        let mut file = 0usize;
        for ch in row.chars() {
            if let Some(skip) = ch.to_digit(10) {
                file += skip as usize;
                continue;
            }
            let rank = 7usize.saturating_sub(row_index);
            let color = if ch.is_ascii_uppercase() {
                PlayerColor::White
            } else {
                PlayerColor::Black
            };
            let kind = match ch.to_ascii_lowercase() {
                'p' => PieceKind::Pawn,
                'n' => PieceKind::Knight,
                'b' => PieceKind::Bishop,
                'r' => PieceKind::Rook,
                'q' => PieceKind::Queen,
                'k' => PieceKind::King,
                _ => continue,
            };
            pieces.push(ChessPieceView {
                kind,
                color,
                square: format!("{}{}", (b'a' + file as u8) as char, rank + 1),
            });
            file += 1;
        }
    }
    pieces
}

pub fn piece_from_chess(piece: Piece) -> PieceKind {
    match piece {
        Piece::Pawn => PieceKind::Pawn,
        Piece::Knight => PieceKind::Knight,
        Piece::Bishop => PieceKind::Bishop,
        Piece::Rook => PieceKind::Rook,
        Piece::Queen => PieceKind::Queen,
        Piece::King => PieceKind::King,
    }
}

pub fn piece_to_chess(piece: PieceKind) -> Piece {
    match piece {
        PieceKind::Pawn => Piece::Pawn,
        PieceKind::Knight => Piece::Knight,
        PieceKind::Bishop => Piece::Bishop,
        PieceKind::Rook => Piece::Rook,
        PieceKind::Queen => Piece::Queen,
        PieceKind::King => Piece::King,
    }
}

pub fn color_from_chess(color: Color) -> PlayerColor {
    match color {
        Color::White => PlayerColor::White,
        Color::Black => PlayerColor::Black,
    }
}

pub fn chess_color_from_player(color: PlayerColor) -> Color {
    match color {
        PlayerColor::White => Color::White,
        PlayerColor::Black => Color::Black,
    }
}

pub fn board_position(square: &str) -> (f32, f32, f32) {
    let bytes = square.as_bytes();
    let file = bytes.first().copied().unwrap_or(b'a').saturating_sub(b'a') as f32;
    let rank = bytes.get(1).copied().unwrap_or(b'1').saturating_sub(b'1') as f32;
    let x = file * crate::constants::TILE_SIZE - crate::constants::BOARD_OFFSET;
    let z = (7.0 - rank) * crate::constants::TILE_SIZE - crate::constants::BOARD_OFFSET;
    (x, 0.0, z)
}

fn captured_square_for(
    board: &Board,
    from_square: Square,
    to_square: Square,
    moved_piece: Piece,
) -> Option<Square> {
    if board.piece_on(to_square).is_some() {
        return Some(to_square);
    }
    if moved_piece == Piece::Pawn && from_square.get_file() != to_square.get_file() {
        return Some(Square::make_square(
            from_square.get_rank(),
            to_square.get_file(),
        ));
    }
    None
}

fn needs_promotion(board: Board, from: Square, to: Square, piece: Piece) -> bool {
    if piece != Piece::Pawn {
        return false;
    }
    let color = board.color_on(from).unwrap_or(Color::White);
    let target_rank = to.get_rank().to_index();
    (color == Color::White && target_rank == 7) || (color == Color::Black && target_rank == 0)
}

fn notation_for(
    piece: Piece,
    color: PlayerColor,
    from: &str,
    to: &str,
    captured: Option<Piece>,
    promotion: Option<PieceKind>,
) -> String {
    let piece_label = piece_from_chess(piece).label();
    let capture = if captured.is_some() { "x" } else { "-" };
    let suffix = promotion
        .map(|kind| format!("={}", kind.label()))
        .unwrap_or_default();
    if piece == Piece::Pawn {
        format!("{from}{capture}{to}{suffix}")
    } else {
        format!("{piece_label}{from}{capture}{to}{suffix}")
    }
    .replace("White", color.label())
}

fn promotion_char(piece: PieceKind) -> char {
    match piece {
        PieceKind::Queen => 'q',
        PieceKind::Rook => 'r',
        PieceKind::Bishop => 'b',
        PieceKind::Knight => 'n',
        PieceKind::Pawn | PieceKind::King => 'q',
    }
}

fn piece_from_promotion_char(ch: char) -> Option<PieceKind> {
    match ch.to_ascii_lowercase() {
        'q' => Some(PieceKind::Queen),
        'r' => Some(PieceKind::Rook),
        'b' => Some(PieceKind::Bishop),
        'n' => Some(PieceKind::Knight),
        _ => None,
    }
}

fn random_puzzle_index() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        ((js_sys::Math::random() * SAMPLE_PUZZLES.len() as f64).floor() as usize)
            .min(SAMPLE_PUZZLES.len() - 1)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_applies_legal_move() {
        let mut game = GameController::default();
        assert!(game.make_move("e2", "e4", None));
        assert_eq!(game.turn, PlayerColor::Black);
        assert_eq!(game.history_uci, vec!["e2e4"]);
    }

    #[test]
    fn rejects_illegal_move() {
        let mut game = GameController::default();
        assert!(!game.make_move("e2", "e5", None));
        assert_eq!(game.fen, START_FEN);
    }

    #[test]
    fn promotion_can_be_completed() {
        let mut game = GameController::default();
        game.board = Board::from_str("8/P7/8/8/8/8/8/4k2K w - - 0 1").unwrap();
        game.sync_status();
        assert!(!game.make_move("a7", "a8", None));
        assert!(game.promotion_pending.is_some());
        assert!(game.set_promotion(PieceKind::Queen));
        assert!(game.fen.starts_with("Q7/8/8/8/8/8/8/4k2K"));
    }

    #[test]
    fn puzzle_rejects_wrong_move() {
        let mut game = GameController::default();
        game.init(GameMode::Puzzles, Difficulty::Easy, "Player");
        assert!(!game.make_move("a2", "a3", None));
        assert!(!game.is_puzzle_solved);
    }

    #[test]
    fn replay_cursor_moves() {
        let mut game = GameController::default();
        assert!(game.make_move("e2", "e4", None));
        game.start_replay();
        game.prev_replay_move();
        assert_eq!(game.replay_index, 0);
        game.next_replay_move();
        assert_eq!(game.replay_index, 1);
    }
}
