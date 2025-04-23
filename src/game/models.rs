use rand::Rng;

pub enum Action {
    MoveLeft,
    MoveRight,
    MoveDown,
    UserQuit
}

pub enum Block {
    L,
    O,
    T,
    I,
    S,
    Z,
    J,
}

impl Block {
    // Function to generate a random block
    pub fn random() -> Block {
        let mut rng = rand::rng();
        match rng.random_range(0..7) {
            0 => Block::L,
            1 => Block::O,
            2 => Block::T,
            3 => Block::I,
            4 => Block::S,
            5 => Block::Z,
            _ => Block::J,
        }
    }

    // Function to get cells of the block in a matrix representation
    pub fn get_matrix(&self) -> Vec<Vec<u8>> {
        match self {
            Block::L => vec![vec![1, 0, 0], vec![1, 1, 1]],
            Block::O => vec![vec![1, 1], vec![1, 1]],
            Block::T => vec![vec![0, 1, 0], vec![1, 1, 1]],
            Block::I => vec![vec![1], vec![1], vec![1], vec![1]],
            Block::S => vec![vec![0, 1, 1], vec![1, 1, 0]],
            Block::Z => vec![vec![1, 1, 0], vec![0, 1, 1]],
            Block::J => vec![vec![0, 0, 1], vec![1, 1, 1]],
        }
    }
}
