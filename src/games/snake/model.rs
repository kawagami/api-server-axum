use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMessage {
    ChangeDirection { direction: Direction },
    Restart,
    Start,
}

#[derive(Serialize)]
pub struct ServerMessage {
    pub snake: Vec<(i32, i32)>,
    pub food: (i32, i32),
    pub score: u32,
    pub game_over: bool,
    pub game_started: bool,
}

#[derive(Copy, Clone, Deserialize, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct SnakeGameState {
    pub snake: Vec<(i32, i32)>,
    pub direction: Direction,
    pub food: (i32, i32),
    pub score: u32,
    pub game_over: bool,
    pub game_started: bool,
}

impl SnakeGameState {
    pub fn new() -> Self {
        Self {
            snake: vec![(5, 5)],
            direction: Direction::Right,
            food: (10, 10),
            score: 0,
            game_over: false,
            game_started: false,
        }
    }

    pub fn update(&mut self) {
        if self.game_over || !self.game_started {
            return;
        }

        // 移動頭部位置
        let (x, y) = self.snake[0];
        let new_head = match self.direction {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        };

        // 撞牆判定（範例用 0~20 範圍）
        if new_head.0 < 0 || new_head.0 > 20 || new_head.1 < 0 || new_head.1 > 20 {
            self.game_over = true;
            return;
        }

        // 撞到自己
        if self.snake.contains(&new_head) {
            self.game_over = true;
            return;
        }

        self.snake.insert(0, new_head);

        if new_head == self.food {
            self.score += 1;
            self.food = (
                rand::random::<u8>() as i32 % 20,
                rand::random::<u8>() as i32 % 20,
            );
        } else {
            self.snake.pop();
        }
    }

    pub fn handle_input(&mut self, msg: ClientMessage) {
        match msg {
            ClientMessage::ChangeDirection { direction } => {
                if !is_opposite(self.direction, direction) {
                    self.direction = direction;
                }
            }
            ClientMessage::Restart => {
                *self = SnakeGameState::new();
                self.game_started = true;
            }
            ClientMessage::Start => {
                self.game_started = true; // 處理開始消息
            }
        }
    }
}

fn is_opposite(d1: Direction, d2: Direction) -> bool {
    matches!(
        (d1, d2),
        (Direction::Up, Direction::Down)
            | (Direction::Down, Direction::Up)
            | (Direction::Left, Direction::Right)
            | (Direction::Right, Direction::Left)
    )
}

impl From<&SnakeGameState> for ServerMessage {
    fn from(state: &SnakeGameState) -> Self {
        ServerMessage {
            snake: state.snake.clone(),
            food: state.food,
            score: state.score,
            game_over: state.game_over,
            game_started: state.game_started,
        }
    }
}
