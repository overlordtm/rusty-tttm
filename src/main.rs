#![deny(warnings)]
use warp::Filter;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Player {
    X,
    O,
}

#[derive(Clone, Debug)]
struct TicTacToe {
    size: usize,
    board: Vec<Vec<Option<Player>>>,  // None represents an empty cell, Some(Player) represents a player's move
    current_turn: Player,
}

impl TicTacToe {
    // Initialize a new N x N Tic-Tac-Toe board
    fn new(size: usize) -> Self {
        Self {
            size,
            board: vec![vec![None; size]; size],  // Empty N x N board
            current_turn: Player::X,  // Player X always goes first
        }
    }

    fn parse_moves(&mut self, moves_str: &str) -> Result<(), &'static str> {
        // Split the string into individual move components (e.g., "X-1-1" and "O-0-0")
        let moves = moves_str.split('_');

        for mv in moves {
            // Split each move into player, row, and column
            let parts: Vec<&str> = mv.split('-').collect();

            if parts.len() != 3 {
                return Err("Invalid move format");
            }

            // Parse the player (X or O)
            let player = match parts[0] {
                "X" => Player::X,
                "O" => Player::O,
                _ => return Err("Invalid player"),
            };

            // Parse row and column
            let row: usize = parts[1].parse().map_err(|_| "Invalid row")?;
            let col: usize = parts[2].parse().map_err(|_| "Invalid column")?;

            // Make the move
            if row >= self.size || col >= self.size {
                return Err("Move out of bounds");
            }
            if self.board[row][col].is_some() {
                return Err("Cell already taken");
            }

            // Place the move on the board
            self.board[row][col] = Some(player);

            // Set the current player
            self.current_turn = match player {
                Player::X => Player::O,
                Player::O => Player::X,
            };
        }

        Ok(())
    }

    // Get the current player
    #[allow(dead_code)]
    fn current_player(&self) -> Player {
        self.current_turn.clone()
    }

    // Make a move at position (row, col)
    #[allow(dead_code)]
    fn make_move(&mut self, row: usize, col: usize) -> Result<(), &'static str> {
        if row >= self.size || col >= self.size {
            return Err("Invalid move: Out of bounds");
        }

        if self.board[row][col].is_some() {
            return Err("Invalid move: Cell already taken");
        }

        self.board[row][col] = Some(self.current_turn.clone());

        // Switch turns
        self.current_turn = match self.current_turn {
            Player::X => Player::O,
            Player::O => Player::X,
        };

        Ok(())
    }

    // Check if the board is full
    fn is_full(&self) -> bool {
        self.board.iter().all(|row| row.iter().all(|cell| cell.is_some()))
    }

    // Check if a player has won
    fn check_winner(&self) -> Option<Player> {
        // Check rows, columns, and diagonals for a win
        for i in 0..self.size {
            // Check row i
            if self.board[i].iter().all(|&cell| cell == Some(Player::X)) {
                return Some(Player::X);
            }
            if self.board[i].iter().all(|&cell| cell == Some(Player::O)) {
                return Some(Player::O);
            }

            // Check column i
            if (0..self.size).all(|j| self.board[j][i] == Some(Player::X)) {
                return Some(Player::X);
            }
            if (0..self.size).all(|j| self.board[j][i] == Some(Player::O)) {
                return Some(Player::O);
            }
        }

        // Check the main diagonal
        if (0..self.size).all(|i| self.board[i][i] == Some(Player::X)) {
            return Some(Player::X);
        }
        if (0..self.size).all(|i| self.board[i][i] == Some(Player::O)) {
            return Some(Player::O);
        }

        // Check the anti-diagonal
        if (0..self.size).all(|i| self.board[i][self.size - 1 - i] == Some(Player::X)) {
            return Some(Player::X);
        }
        if (0..self.size).all(|i| self.board[i][self.size - 1 - i] == Some(Player::O)) {
            return Some(Player::O);
        }

        None
    }

    // Evaluate the board state (for min-max algorithm)
    fn evaluate(&self) -> i32 {
        match self.check_winner() {
            Some(Player::X) => 1,  // X wins
            Some(Player::O) => -1, // O wins
            None => 0,             // Draw or game not finished
        }
    }

    // Get available moves (empty cells)
    fn available_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for row in 0..self.size {
            for col in 0..self.size {
                if self.board[row][col].is_none() {
                    moves.push((row, col));
                }
            }
        }
        moves
    }

     /// Run the Min-Max algorithm with alpha-beta pruning
    /// Returns the best score and the best move (row, col)
    fn minmax(
        &mut self,
        depth: usize,  // Depth of the recursion
        player: Player,  // Whether it's the maximizing player (X) or minimizing player (O)
        alpha: i32,  // Alpha value
        beta: i32,   // Beta value
    ) -> (i32, Option<(usize, usize)>) {
        // Evaluate the current board state
        let score = self.evaluate();
        if score == 1 || score == -1 || self.is_full() {
            // If game is won or full, return the evaluation score and no move
            return (score, None);
        }

        // Initialize alpha and beta values for pruning
        let mut alpha = alpha;
        let mut beta = beta;
        let mut best_move = None;

        if player == Player::X {
            // Maximizing player (X)
            let mut max_eval = i32::MIN;

            // Iterate over all available moves
            for (row, col) in self.available_moves() {
                // Make the move
                self.board[row][col] = Some(Player::X);
                
                // Recurse
                let (eval, _) = self.minmax(depth + 1, Player::O, alpha, beta);

                // Undo the move
                self.board[row][col] = None;

                // Update max evaluation
                if eval > max_eval {
                    max_eval = eval;
                    best_move = Some((row, col));
                }

                // Alpha-beta pruning
                alpha = alpha.max(eval);
                if beta <= alpha {
                    break;  // Beta cutoff
                }
            }

            return (max_eval, best_move);

        } else {
            // Minimizing player (O)
            let mut min_eval = i32::MAX;

            // Iterate over all available moves
            for (row, col) in self.available_moves() {
                // Make the move
                self.board[row][col] = Some(Player::O);
                
                // Recurse
                let (eval, _) = self.minmax(depth + 1, Player::X, alpha, beta);

                // Undo the move
                self.board[row][col] = None;

                // Update min evaluation
                if eval < min_eval {
                    min_eval = eval;
                    best_move = Some((row, col));
                }

                // Alpha-beta pruning
                beta = beta.min(eval);
                if beta <= alpha {
                    break;  // Alpha cutoff
                }
            }

            return (min_eval, best_move);
        }
    }

    // Function to draw the Tic Tac Toe board in ASCII
    #[allow(dead_code)]
    fn draw_board(&self) {
        for row in 0..self.size {
            for col in 0..self.size {
                // Get the current cell
                let cell = self.board[row][col];
                // Print the cell content or a placeholder
                let symbol = match cell {
                    Some(Player::X) => 'X',
                    Some(Player::O) => 'O',
                    None => ' ',
                };
                print!("{} ", symbol); // Print the symbol with space
            }
            println!(); // New line after each row
            // Print a separator line between rows
            if row < self.size - 1 {
                println!("{}", "-".repeat(self.size * 2 - 1)); // Adjust separator length
            }
        }
        println!(); // Extra line for better readability
    }

}

// The GET /move request
// The game server will pass the following URL query parameters to the player server.

//     gid - UUID that represents the given game ID.
//     size - The size of tic-tac-toe grid.
//         By default, the size is set to 3, representing the grid of size 3x3.
//         Possible sizes are 3, 5 and 7.
//         On grids with size 5 or 7 - four symbols are needed for a win.
//     playing - A symbol that the player server needs to play.
//         Possible values are X or O.
//     moves - A string that represents the previous moves.
//         Moves are separated by _ and positions by -.
//         Example: X-1-1_O-0-0 means that the X symbol was at location 1,1 (centre of grid) and O at 0,0 (top-left corner of the grid).
#[derive(Deserialize, Debug)]
struct MoveParams {
    gid: Uuid,
    size: u32,
    playing: String,
    moves: String,
}

async fn get_move(params: MoveParams) -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("Received request: gid:{:?} size:{:?} playing:{:?} moves:{:?}", params.gid, params.size, params.playing, params.moves);

    let mut ttt = TicTacToe::new(params.size as usize);

    match ttt.parse_moves(&params.moves) {
        Err(err) => {
            log::error!("parse_moves error: {}", err);
            return Ok("Error:Sorry. Can't do it bro.".to_string());
        }
        Ok(_) => {
            let player = match params.playing.as_str() {
                "X" => Player::X,
                "O" => Player::O,
                _ => {
                    log::error!("Invalid player: {}", params.playing);
                    return Ok("Error:Sorry. Can't do it bro.".to_string());
                }
            };

            let (_, best_move) = ttt.minmax(0, player, i32::MIN, i32::MAX);
            
            if let Some((row, col)) = best_move {
                log::info!("Best move: row:{:?} col:{:?}", row, col);
                // let res = ttt.make_move(row, col);
                // ttt.draw_board();
                let txt = format!("Move:{}-{}-{}", params.playing, row, col);
                return Ok(txt);
            } else {
                log::error!("No best move found");
                return Ok("Sorry. Can't do it bro.".to_string());
            }

        }
    }

}

#[tokio::main]
async fn main() {
    // Initialize the logger
    env_logger::init();

    let routes = warp::path("move")
        .and(warp::get())
        .and(warp::query::<MoveParams>())
        .and_then(get_move);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}