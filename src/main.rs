// Import required dependencies
use ggez::{Context, GameResult, graphics::{self, DrawParam, Color, Canvas, Text}, event, input::keyboard::KeyCode, timer};
use rand::Rng;
use std::time::Duration;

// Game constants
const BLOCK_SIZE: f32 = 25.0;
const GRID_WIDTH: i32 = 10;
const GRID_HEIGHT: i32 = 20;
const SCREEN_WIDTH: f32 = BLOCK_SIZE * GRID_WIDTH as f32;
const SCREEN_HEIGHT: f32 = BLOCK_SIZE * GRID_HEIGHT as f32;
const MOVE_INTERVAL: Duration = Duration::from_millis(100); // Minimum time between moves
const DROP_INTERVAL: Duration = Duration::from_millis(500); // Time between automatic drops

// Represents a single block in the game
#[derive(Clone, Copy)]
struct Block {
    x: i32,
    y: i32,
    color: Color,
}

// Represents a complete tetromino (a game piece made up of 4 blocks)
struct Tetromino {
    blocks: Vec<Block>,
    block_type: i32, // Used to identify the shape type (0-6)
}

// Main game state structure
struct GameState {
    tetromino: Tetromino,      // Current falling piece
    grid: Vec<Vec<Option<Color>>>, // Game grid: None = empty, Some(Color) = filled
    game_over: bool,           // Game over flag
    score: i32,                // Current score
    last_move_time: Duration,  // Time of last movement
    last_drop_time: Duration,  // Time of last automatic drop
    last_rotate_time: Duration, // Time of last rotation
}

impl GameState {
    // Initialize a new game state
    fn new() -> Self {
        let grid = vec![vec![None; GRID_WIDTH as usize]; GRID_HEIGHT as usize];
        let tetromino = Self::create_random_tetromino();
        
        GameState {
            tetromino,
            grid,
            game_over: false,
            score: 0,
            last_move_time: Duration::ZERO,
            last_drop_time: Duration::ZERO,
            last_rotate_time: Duration::ZERO,
        }
    }

    // Create a new random tetromino piece
    fn create_random_tetromino() -> Tetromino {
        let mut rng = rand::thread_rng();
        let block_type = rng.gen_range(0..7);
        let (blocks, color) = match block_type {
            0 => (// I-shape
                vec![(3,0), (4,0), (5,0), (6,0)],
                Color::CYAN),
            1 => (// Square
                vec![(4,0), (5,0), (4,1), (5,1)],
                Color::YELLOW),
            2 => (// L-shape
                vec![(3,0), (3,1), (4,1), (5,1)],
                Color::RED),
            3 => (// J-shape
                vec![(5,0), (3,1), (4,1), (5,1)],
                Color::GREEN),
            4 => (// T-shape
                vec![(4,0), (3,1), (4,1), (5,1)],
                Color::MAGENTA),
            5 => (// S-shape
                vec![(4,0), (5,0), (3,1), (4,1)],
                Color::WHITE),
            _ => (// Z-shape
                vec![(3,0), (4,0), (4,1), (5,1)],
                Color::new(1.0, 0.5, 0.0, 1.0)), // Orange
        };
        
        Tetromino {
            blocks: blocks.into_iter()
                        .map(|(x, y)| Block { x, y, color })
                        .collect(),
            block_type,
        }
    }

    // Rotate the current tetromino 90 degrees clockwise
    fn rotate_tetromino(&mut self) {
        if self.tetromino.block_type == 1 { // Square doesn't need rotation
            return;
        }

        let center = self.tetromino.blocks[1]; // Use second block as rotation center
        let mut new_blocks = Vec::new();

        for block in &self.tetromino.blocks {
            // Calculate new position after rotation
            let dx = block.x - center.x;
            let dy = block.y - center.y;
            let new_x = center.x - dy;
            let new_y = center.y + dx;

            // Check if rotation is valid
            if new_x < 0 || new_x >= GRID_WIDTH || new_y >= GRID_HEIGHT {
                return;
            }
            if new_y >= 0 && self.grid[new_y as usize][new_x as usize].is_some() {
                return;
            }
            new_blocks.push(Block {
                x: new_x,
                y: new_y,
                color: block.color,
            });
        }

        self.tetromino.blocks = new_blocks;
    }

    // Move the current tetromino by the specified amount
    fn move_tetromino(&mut self, dx: i32, dy: i32) {
        let mut can_move = true;
        // Check if the move is valid
        for block in &self.tetromino.blocks {
            let new_x = block.x + dx;
            let new_y = block.y + dy;
            
            if new_x < 0 || new_x >= GRID_WIDTH || new_y >= GRID_HEIGHT {
                can_move = false;
                break;
            }
            
            if new_y >= 0 && self.grid[new_y as usize][new_x as usize].is_some() {
                can_move = false;
                break;
            }
        }

        if can_move {
            // Perform the move
            for block in &mut self.tetromino.blocks {
                block.x += dx;
                block.y += dy;
            }
        } else if dy > 0 {
            // If we can't move down, freeze the tetromino
            self.freeze_tetromino();
        }
    }

    // Freeze the current tetromino in place and create a new one
    fn freeze_tetromino(&mut self) {
        for block in &self.tetromino.blocks {
            if block.y >= 0 {
                self.grid[block.y as usize][block.x as usize] = Some(block.color);
            } else {
                self.game_over = true;
                return;
            }
        }
        self.clear_lines();
        self.tetromino = Self::create_random_tetromino();
    }

    // Check for and clear completed lines
    fn clear_lines(&mut self) {
        let mut lines_cleared = 0;
        let mut y = GRID_HEIGHT - 1;
        while y >= 0 {
            if self.grid[y as usize].iter().all(|cell| cell.is_some()) {
                lines_cleared += 1;
                // Move all lines above down
                for row in (1..=y).rev() {
                    self.grid[row as usize] = self.grid[(row - 1) as usize].clone();
                }
                self.grid[0] = vec![None; GRID_WIDTH as usize];
            } else {
                y -= 1;
            }
        }

        // Calculate score based on number of lines cleared
        match lines_cleared {
            1 => self.score += 100,
            2 => self.score += 300,
            3 => self.score += 500,
            4 => self.score += 800,
            _ => (),
        }
    }
}

// Implement the game loop handlers
impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.game_over {
            return Ok(());
        }

        let now = timer::time_since_start(ctx);

        // Handle left/right movement
        if now - self.last_move_time >= MOVE_INTERVAL {
            if ctx.keyboard.is_key_pressed(KeyCode::Left) {
                self.move_tetromino(-1, 0);
                self.last_move_time = now;
            }
            if ctx.keyboard.is_key_pressed(KeyCode::Right) {
                self.move_tetromino(1, 0);
                self.last_move_time = now;
            }
        }

        // Handle fast drop
        if ctx.keyboard.is_key_pressed(KeyCode::Down) {
            if now - self.last_move_time >= MOVE_INTERVAL {
                self.move_tetromino(0, 1);
                self.last_move_time = now;
            }
        }

        // Handle rotation
        if ctx.keyboard.is_key_pressed(KeyCode::Up) {
            if now - self.last_rotate_time >= MOVE_INTERVAL {
                self.rotate_tetromino();
                self.last_rotate_time = now;
            }
        }

        // Handle automatic dropping
        if now - self.last_drop_time >= DROP_INTERVAL {
            self.move_tetromino(0, 1);
            self.last_drop_time = now;
        }

        Ok(())
    }

    // Draw the game state
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        // Draw the current tetromino
        for block in &self.tetromino.blocks {
            let rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(
                    block.x as f32 * BLOCK_SIZE,
                    block.y as f32 * BLOCK_SIZE,
                    BLOCK_SIZE - 1.0,
                    BLOCK_SIZE - 1.0,
                ),
                block.color,
            )?;
            canvas.draw(&rect, DrawParam::default());
        }

        // Draw the frozen blocks
        for (y, row) in self.grid.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if let Some(color) = cell {
                    let rect = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new(
                            x as f32 * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            BLOCK_SIZE - 1.0,
                            BLOCK_SIZE - 1.0,
                        ),
                        *color,
                    )?;
                    canvas.draw(&rect, DrawParam::default());
                }
            }
        }

        // Draw the score
        let score_text = Text::new(format!("Score: {}", self.score));
        canvas.draw(
            &score_text,
            DrawParam::default()
                .dest([10.0, 10.0])
                .color(Color::WHITE),
        );

        // Draw game over message if applicable
        if self.game_over {
            let game_over_text = Text::new("Game Over!");
            canvas.draw(
                &game_over_text,
                DrawParam::default()
                    .dest([SCREEN_WIDTH / 2.0 - 40.0, SCREEN_HEIGHT / 2.0])
                    .color(Color::RED),
            );
        }

        canvas.finish(ctx)?;
        Ok(())
    }
}

// Main function to set up and run the game
fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("tetris", "cascade")
        .window_setup(ggez::conf::WindowSetup::default().title("Tetris"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT));
    
    let (ctx, event_loop) = cb.build()?;
    let state = GameState::new();
    event::run(ctx, event_loop, state)
}