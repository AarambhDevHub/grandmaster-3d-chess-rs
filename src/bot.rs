use chess::{Board, BoardStatus, ChessMove, Color, File, MoveGen, Piece, Rank, Square};

use crate::types::Difficulty;

pub fn choose_bot_move(board: &Board, difficulty: Difficulty) -> Option<ChessMove> {
    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    if moves.is_empty() {
        return None;
    }

    order_moves(board, &mut moves);

    match difficulty {
        Difficulty::Easy => Some(choose_easy(board, &moves)),
        Difficulty::Medium => Some(choose_humanized_search(board, &moves, BotStyle::medium())),
        Difficulty::Hard => Some(choose_humanized_search(board, &moves, BotStyle::hard())),
    }
}

pub fn evaluate_board(board: &Board) -> i32 {
    match board.status() {
        BoardStatus::Checkmate => {
            return if board.side_to_move() == Color::White {
                -100_000
            } else {
                100_000
            };
        }
        BoardStatus::Stalemate => return 0,
        BoardStatus::Ongoing => {}
    }

    let mut score = 0;
    for rank in 0..8 {
        for file in 0..8 {
            let square = Square::make_square(Rank::from_index(rank), File::from_index(file));
            let Some(piece) = board.piece_on(square) else {
                continue;
            };
            let Some(color) = board.color_on(square) else {
                continue;
            };
            let mut value = piece_value(piece);
            if (2..=5).contains(&rank) && (2..=5).contains(&file) {
                value += 4;
            }
            if matches!(piece, Piece::Knight | Piece::Bishop) && (2..=5).contains(&rank) {
                value += 2;
            }
            if color == Color::White {
                score += value;
            } else {
                score -= value;
            }
        }
    }

    let mobility = MoveGen::new_legal(board).len() as i32;
    if board.side_to_move() == Color::White {
        score += mobility;
    } else {
        score -= mobility;
    }

    score
}

fn minimax(board: &Board, depth: u8, mut alpha: i32, mut beta: i32, maximizing: bool) -> i32 {
    if depth == 0 || board.status() != BoardStatus::Ongoing {
        return evaluate_board(board);
    }

    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    order_moves(board, &mut moves);

    if maximizing {
        let mut value = i32::MIN + 1;
        for mv in moves {
            let next = board.make_move_new(mv);
            value = value.max(minimax(&next, depth - 1, alpha, beta, false));
            alpha = alpha.max(value);
            if beta <= alpha {
                break;
            }
        }
        value
    } else {
        let mut value = i32::MAX - 1;
        for mv in moves {
            let next = board.make_move_new(mv);
            value = value.min(minimax(&next, depth - 1, alpha, beta, true));
            beta = beta.min(value);
            if beta <= alpha {
                break;
            }
        }
        value
    }
}

#[derive(Clone, Copy)]
struct BotStyle {
    depth: u8,
    best_chance: f64,
    second_chance: f64,
    third_chance: f64,
    mistake_chance: f64,
    pool: usize,
}

impl BotStyle {
    fn medium() -> Self {
        Self {
            depth: 1,
            best_chance: 0.34,
            second_chance: 0.28,
            third_chance: 0.18,
            mistake_chance: 0.24,
            pool: 6,
        }
    }

    fn hard() -> Self {
        Self {
            depth: 2,
            best_chance: 0.48,
            second_chance: 0.25,
            third_chance: 0.13,
            mistake_chance: 0.12,
            pool: 4,
        }
    }
}

fn choose_humanized_search(board: &Board, moves: &[ChessMove], style: BotStyle) -> ChessMove {
    let mut ranked = ranked_moves(board, moves, style.depth);
    if ranked.is_empty() {
        return moves[0];
    }
    if random_unit() < style.mistake_chance && ranked.len() > 3 {
        return choose_soft_mistake(&ranked, style.pool);
    }

    let pool = style.pool.min(ranked.len()).max(1);
    ranked.truncate(pool);
    let roll = random_unit();
    let index = if roll < style.best_chance || ranked.len() == 1 {
        0
    } else if roll < style.best_chance + style.second_chance || ranked.len() == 2 {
        1
    } else if roll < style.best_chance + style.second_chance + style.third_chance
        || ranked.len() == 3
    {
        2
    } else {
        3 + random_index(ranked.len().saturating_sub(3))
    };
    ranked[index.min(ranked.len() - 1)].0
}

fn ranked_moves(board: &Board, moves: &[ChessMove], depth: u8) -> Vec<(ChessMove, i32)> {
    let side = board.side_to_move();
    let mut ranked: Vec<(ChessMove, i32)> = moves
        .iter()
        .copied()
        .map(|mv| {
            let next = board.make_move_new(mv);
            let raw = if depth <= 1 {
                evaluate_board(&next)
            } else {
                minimax(
                    &next,
                    depth - 1,
                    i32::MIN + 1,
                    i32::MAX - 1,
                    next.side_to_move() == Color::White,
                )
            };
            let perspective = if side == Color::White { raw } else { -raw };
            (mv, perspective + human_tactical_bonus(board, mv))
        })
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1));
    ranked
}

fn choose_soft_mistake(ranked: &[(ChessMove, i32)], pool: usize) -> ChessMove {
    let start = pool.min(ranked.len().saturating_sub(1));
    let end = (start + 5).min(ranked.len());
    if start >= end {
        return ranked[random_index(ranked.len())].0;
    }
    ranked[start + random_index(end - start)].0
}

fn human_tactical_bonus(board: &Board, mv: ChessMove) -> i32 {
    let mut score = 0;
    if let Some(captured) = board.piece_on(mv.get_dest()) {
        score += piece_value(captured) / 6;
    }
    if let Some(promotion) = mv.get_promotion() {
        score += piece_value(promotion) / 5;
    }
    let next = board.make_move_new(mv);
    if next.checkers().popcnt() > 0 {
        score += 18;
    }
    if next.status() == BoardStatus::Checkmate {
        score += 20_000;
    }
    score
}

fn choose_easy(board: &Board, moves: &[ChessMove]) -> ChessMove {
    let captures: Vec<ChessMove> = moves
        .iter()
        .copied()
        .filter(|mv| board.piece_on(mv.get_dest()).is_some())
        .collect();
    if !captures.is_empty() && random_unit() > 0.55 {
        captures[random_index(captures.len())]
    } else {
        moves[random_index(moves.len())]
    }
}

fn order_moves(board: &Board, moves: &mut [ChessMove]) {
    moves.sort_by_key(|mv| -move_score(board, *mv));
}

fn move_score(board: &Board, mv: ChessMove) -> i32 {
    let mut score = 0;
    if let Some(captured) = board.piece_on(mv.get_dest()) {
        score += piece_value(captured) * 10;
    }
    if let Some(promotion) = mv.get_promotion() {
        score += piece_value(promotion) * 4;
    }
    let next = board.make_move_new(mv);
    if next.status() == BoardStatus::Checkmate {
        score += 50_000;
    }
    score
}

fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 20_000,
    }
}

fn random_index(len: usize) -> usize {
    if len <= 1 {
        return 0;
    }
    ((random_unit() * len as f64).floor() as usize).min(len - 1)
}

fn random_unit() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Math::random()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0.37
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bot_returns_a_legal_move() {
        let board = Board::default();
        let mv = choose_bot_move(&board, Difficulty::Medium).unwrap();
        assert!(MoveGen::new_legal(&board).any(|legal| legal == mv));
    }
}
