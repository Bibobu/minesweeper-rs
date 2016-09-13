use rand;
use rand::Rng;
use std::collections::VecDeque;
use common::{ParamType, MoveDestination};
use piston_window::*;

pub enum Content {
    Number(u8),
    Mine(bool), // bool is true when this is the mine that caused you to loose the game.
    None,
}

struct Cell {
    content: Content,
    revealed: bool,
    marked: bool,
}

impl Cell {
    fn create_empty() -> Cell {
        Cell {
            content: Content::None,
            revealed: false,
            marked: false,
        }
    }

    fn create_with_mine() -> Cell {
        Cell {
            content: Content::Mine(false),
            revealed: false,
            marked: false,
        }
    }

    pub fn clear(&mut self) {
        self.content = Content::None;
        self.revealed = false;
        self.marked = false;
    }

    fn reveal(&mut self) -> &Content {
        self.revealed = true;
        &self.content
    }

    fn toggle_mark(&mut self) {
        self.marked = !self.marked;
    }
}

pub struct Field {
    cells: Vec<Cell>,
    width: u32,
    height: u32,
    mines: u32,
    size: u32,
    selected_x: u32,
    selected_y: u32,
    nubmers_total: u32,
    nubmers_opened: u32,
    need_regen: bool,
}

impl Field {
    pub fn new(width: u32, height: u32, mines: u32) -> Field {
        let mut field = Field {
            width: width,
            height: height,
            cells: vec![],
            mines: mines,
            size: width * height,
            selected_x: width / 2,
            selected_y: height / 2,
            nubmers_total: 0,
            nubmers_opened: 0,
            need_regen: true,
        };
        field.reinit_vec();
        field
    }

    pub fn restart(&mut self) {
        self.need_regen = true;
        self.reinit_vec();
    }

    fn reinit_vec(&mut self) {
        self.cells.clear();
        self.size = self.width * self.height;
        for _i in 0..self.size {
            self.cells.push(Cell::create_empty());
        }
        self.selected_x = self.width / 2;
        self.selected_y = self.height / 2;
    }

    fn reset(&mut self, cursor_ind: u32) {
        self.clear();

        let cursor_near_cells = self.get_near_cells_ids(cursor_ind);

        // reinit cells with mines
        let mut new_cells = Vec::new();
        for i in 0..self.size - cursor_near_cells.len() as u32 {
            let cell = if i < self.mines {
                Cell::create_with_mine()
            } else {
                Cell::create_empty()
            };

            new_cells.push(cell);
        }

        // shuffle them
        let mut rng = rand::thread_rng();
        rng.shuffle(&mut new_cells);

        // push empty cells near with cursor to avoid mines in this positions
        for ind in cursor_near_cells {
            new_cells.insert(ind as usize, Cell::create_empty());
        }

        self.cells = new_cells;

        self.write_numbers_near_mines();
        self.need_regen = false;
    }

    fn get_near_cells_ids(&mut self, ind: u32) -> Vec<u32> {
        let (begin_x, end_x, begin_y, end_y) = self.get_3x3_bounds(ind);

        let mut cursor_neighbor_cells = Vec::new();
        for yi in begin_y..end_y + 1 {
            for xi in begin_x..end_x + 1 {
                let ind = self.get_cell_index(xi, yi);
                cursor_neighbor_cells.push(ind);
            }
        }

        cursor_neighbor_cells
    }

    fn write_numbers_near_mines(&mut self) {
        let mut i: i32 = -1;
        let w = self.width as i32;
        while i < (self.size - 1) as i32 {
            i += 1;
            match self.get_content_safe(i) {
                Some(&Content::None) => {
                    let ct_mine = |b| {
                        match b {
                            true => 1,
                            false => 0,
                        }
                    };
                    // don`t care about row
                    let mut ct = ct_mine(self.is_mine_safe(i - w)) +
                                 ct_mine(self.is_mine_safe(i + w));
                    if i % w > 0 {
                        // check left side position
                        ct += ct_mine(self.is_mine_safe(i - w - 1)) +
                              ct_mine(self.is_mine_safe(i - 1)) +
                              ct_mine(self.is_mine_safe(i + w - 1));
                    }
                    if i % w < w - 1 {
                        // check right side position
                        ct += ct_mine(self.is_mine_safe(i - w + 1)) +
                              ct_mine(self.is_mine_safe(i + 1)) +
                              ct_mine(self.is_mine_safe(i + w + 1));
                    }
                    if ct > 0 {
                        self.get_cell_mut(i as u32).content = Content::Number(ct);
                        self.nubmers_total += 1;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn get_cell_index(&mut self, x: u32, y: u32) -> u32 {
        x + y * self.width
    }

    // return coordinates of cell (x, y) by index in field
    pub fn get_coord(&mut self, ind: u32) -> (u32, u32) {
        (ind % self.width, ind / self.width)
    }

    pub fn get_neighborhood_count(&mut self, ind: u32) -> u8 {
        let (begin_x, end_x, begin_y, end_y) = self.get_3x3_bounds(ind);

        let mut marked_count = 0u8;
        for yi in begin_y..end_y + 1 {
            for xi in begin_x..end_x + 1 {
                let ind = self.get_cell_index(xi, yi);
                if self.marked(ind) {
                    marked_count += 1;
                }
            }
        }

        marked_count
    }

    // return (begin_x, end_x, begin_y, end_y) for specified position in field
    fn get_3x3_bounds(&mut self, ind: u32) -> (u32, u32, u32, u32) {
        let (x, y) = self.get_coord(ind);

        let begin_x = if x > 0 {
            x - 1
        } else {
            x
        };
        let end_x = if x + 1 < self.width {
            x + 1
        } else {
            x
        };
        let begin_y = if y > 0 {
            y - 1
        } else {
            y
        };
        let end_y = if y + 1 < self.height {
            y + 1
        } else {
            y
        };

        (begin_x, end_x, begin_y, end_y)
    }

    fn clear(&mut self) {
        for i in 0..self.size {
            self.get_cell_mut(i).clear();
        }
        self.nubmers_opened = 0;
        self.nubmers_total = 0;
    }

    pub fn reveal(&mut self, i: u32) -> &Content {
        if !self.revealed(i) {
            if let &Content::Number(_i) = self.get_cell_mut(i).reveal() {
                self.nubmers_opened += 1;
            }
        }
        &self.get_content(i)
    }

    pub fn revealed(&self, i: u32) -> bool {
        self.get_cell(i).revealed
    }

    pub fn marked(&self, i: u32) -> bool {
        self.get_cell(i).marked
    }

    pub fn set_killer(&mut self, i: u32) {
        self.get_cell_mut(i).content = Content::Mine(true);
    }

    fn get_cell_mut(&mut self, i: u32) -> &mut Cell {
        self.cells
            .get_mut(i as usize)
            .unwrap_or_else(|| panic!("Range check error at Field::get_cell_mut ({})", i))
    }

    fn get_cell(&self, i: u32) -> &Cell {
        self.cells
            .get(i as usize)
            .unwrap_or_else(|| panic!("Range check error at Field::get_cell ({})", i))
    }

    fn get_content_safe(&self, i: i32) -> Option<&Content> {
        if (i < 0) || ((i as u32) >= self.size) {
            None
        } else {
            Some(self.get_content(i as u32))
        }
    }

    pub fn get_content(&self, i: u32) -> &Content {
        &self.get_cell(i).content
    }

    pub fn count_marked(&self) -> u32 {
        (0..self.size).fold(0, |acc, i: u32| {
            if self.marked(i) {
                acc + 1
            } else {
                acc
            }
        })
    }

    fn is_mine_safe(&self, i: i32) -> bool {
        match self.get_content_safe(i) {
            Some(&Content::Mine(_)) => true,
            _ => false,
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    pub fn reset_if_need(&mut self, cursor_ind: u32) {
        if self.need_regen {
            self.reset(cursor_ind);
        }
    }

    pub fn reveal_all(&mut self) {
        for i in 0..self.size {
            self.get_cell_mut(i).revealed = true;
        }
    }

    pub fn chain_reveal(&mut self, u: u32) {
        if self.marked(u) {
            return;
        }
        let w = self.width as i32;
        // clojure to check for blank cells
        let mut check = |x, d: &mut VecDeque<i32>| {
            match self.get_content_safe(x) {
                Some(&Content::None) => {
                    if !self.revealed(x as u32) {
                        d.push_back(x);
                        self.get_cell_mut(x as u32).marked = false;
                        self.reveal(x as u32);
                    }
                }
                Some(&Content::Number(_n)) => {
                    if !(self.revealed(x as u32)) {
                        self.get_cell_mut(x as u32).marked = false;
                        self.reveal(x as u32);
                    }
                }
                _ => {}
            }
        };
        // BFS initialize
        let deq = &mut VecDeque::new();
        deq.push_back(u as i32);
        // BFS
        while !deq.is_empty() {
            let i = deq.pop_front().unwrap();
            // don`t care about row
            check(i - w, deq);
            check(i + w, deq);
            if i % w > 0 {
                // check left side position
                check(i - w - 1, deq);
                check(i - 1, deq);
                check(i + w - 1, deq);
            }
            if i % w < w - 1 {
                // check right side position
                check(i - w + 1, deq);
                check(i + 1, deq);
                check(i + w + 1, deq);
            }
        }
    }

    pub fn draw(&mut self,
                context: Context,
                graphics: &mut G2d,
                field_rect: [u32; 4],
                glyps: &mut Glyphs) {
        let cell_w = field_rect[2] / self.get_width();
        let cell_h = field_rect[3] / self.get_height();
        for i in 0..self.get_width() {
            for j in 0..self.get_height() {
                let ind = i + j * self.get_width();
                let transform = context.transform.trans((field_rect[0] + i * cell_w) as f64 + 5.0,
                                                        (field_rect[1] + (j + 1) * cell_h) as f64 -
                                                        5.0);
                if self.revealed(ind) {
                    match *self.get_content(i + j * self.get_width()) {
                        Content::Mine(killer) => {
                            rectangle([1.0, 0.0, 0.0, 1.0],
                                      [(field_rect[0] + i * cell_w) as f64,
                                       (field_rect[1] + j * cell_h) as f64,
                                       cell_w as f64,
                                       cell_h as f64],
                                      context.transform,
                                      graphics);
                            if killer {
                                text([1.0, 1.0, 1.0, 1.0],
                                     cell_h,
                                     "*",
                                     glyps,
                                     transform,
                                     graphics);
                            }
                        }
                        Content::Number(n) => {
                            rectangle(color::hex("c0c0c0"),
                                      [(field_rect[0] + i * cell_w) as f64,
                                       (field_rect[1] + j * cell_h) as f64,
                                       cell_w as f64,
                                       cell_h as f64],
                                      context.transform,
                                      graphics);

                            let text_color = match n {
                                1 => color::hex("0000ff"),
                                2 => color::hex("008000"),
                                3 => color::hex("ff0000"),
                                4 => color::hex("000080"),
                                5 => color::hex("800000"),
                                6 => color::hex("008080"),
                                7 => color::hex("000000"),
                                8 => color::hex("808080"),
                                _ => [0.0, 0.0, 0.0, 1.0],
                            };
                            text(text_color,
                                 cell_h,
                                 &*n.to_string(),
                                 glyps,
                                 transform,
                                 graphics);
                        }
                        Content::None => {
                            rectangle(color::hex("c0c0c0"),
                                      [(field_rect[0] + i * cell_w) as f64,
                                       (field_rect[1] + j * cell_h) as f64,
                                       cell_w as f64,
                                       cell_h as f64],
                                      context.transform,
                                      graphics);
                        }
                    }
                } else {
                    // non revealed
                    rectangle(color::hex("303030"),
                              [(field_rect[0] + i * cell_w) as f64,
                               (field_rect[1] + j * cell_h) as f64,
                               cell_w as f64,
                               cell_h as f64],
                              context.transform,
                              graphics);
                }
                if self.marked(ind) {
                    rectangle([0.0, 1.0, 0.0, 0.75],
                              [(field_rect[0] + i * cell_w) as f64,
                               (field_rect[1] + j * cell_h) as f64,
                               cell_w as f64,
                               cell_h as f64],
                              context.transform,
                              graphics);
                }
            }
        }
        rectangle([0.5, 0.5, 0.5, 0.75],
                  [(field_rect[0] + self.selected_x * cell_w) as f64,
                   (field_rect[1] + self.selected_y * cell_h) as f64,
                   cell_w as f64,
                   cell_h as f64],
                  context.transform,
                  graphics);
        for i in 0..self.get_width() + 1 {
            line([0.5, 0.5, 0.5, 1.0],
                 1.0,
                 [(field_rect[0] + i * cell_w) as f64,
                  field_rect[1] as f64,
                  (field_rect[0] + i * cell_w) as f64,
                  (field_rect[1] + field_rect[3]) as f64],
                 context.transform,
                 graphics);
        }
        for i in 0..self.get_height() + 1 {
            line([0.5, 0.5, 0.5, 1.0],
                 1.0,
                 [field_rect[0] as f64,
                  (field_rect[1] + i * cell_h) as f64,
                  (field_rect[0] + field_rect[2]) as f64,
                  (field_rect[1] + i * cell_h) as f64],
                 context.transform,
                 graphics);
        }
    }

    pub fn move_selection(&mut self, dest: MoveDestination) {
        match dest {
            MoveDestination::Up => {
                if self.selected_y > 0 {
                    self.selected_y -= 1;
                }
            }
            MoveDestination::Down => {
                if self.selected_y < self.height - 1 {
                    self.selected_y += 1;
                }
            }
            MoveDestination::Left => {
                if self.selected_x > 0 {
                    self.selected_x -= 1;
                }
            }
            MoveDestination::Right => {
                if self.selected_x < self.width - 1 {
                    self.selected_x += 1;
                }
            }
        }
    }

    pub fn get_selected_ind(&mut self) -> u32 {
        let x = self.selected_x;
        let y = self.selected_y;
        self.get_cell_index(x, y)
    }

    pub fn is_victory(&self) -> bool {
        self.nubmers_total == self.nubmers_opened
    }

    pub fn reinit_field(&mut self, num: u32, param: ParamType) {
        let mut restart_neded = false;
        match param {
            ParamType::Height => {
                if self.height != num {
                    self.height = num;
                    self.reinit_vec();
                    restart_neded = true;
                }
            }
            ParamType::Width => {
                if self.width != num {
                    self.width = num;
                    self.reinit_vec();
                    restart_neded = true;
                }
            }
            ParamType::Mines => {
                if self.mines != num {
                    self.mines = num;
                    restart_neded = true;
                }
            }
        }
        if restart_neded {
            self.restart();
        }
    }

    pub fn toggle_mark(&mut self, i: u32) {
        self.get_cell_mut(i).toggle_mark();
    }

    pub fn total_mines(&self) -> u32 {
        self.mines
    }
}
