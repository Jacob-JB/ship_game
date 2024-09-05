use std::collections::VecDeque;

use bevy::prelude::*;

pub fn build(app: &mut App) {
    app.init_resource::<ShipModuleGrid>();
}

/// Server side resource that contains the locations of modules in the grid
#[derive(Resource, Debug)]
pub struct ShipModuleGrid {
    /// two dimensional grid of entities
    grid: VecDeque<VecDeque<Option<Entity>>>,
    bound: IRect,
}

impl Default for ShipModuleGrid {
    fn default() -> Self {
        ShipModuleGrid {
            grid: VecDeque::new(),
            bound: IRect::EMPTY,
        }
    }
}

impl ShipModuleGrid {
    /// Returns `true` if the grid contains an index
    pub fn contains(&self, index: IVec2) -> bool {
        index.cmpge(self.bound.min).all() && index.cmplt(self.bound.max).all()
    }

    pub fn get(&self, index: IVec2) -> Option<Entity> {
        if !self.contains(index) {
            return None;
        }

        let offset = index - self.bound.min;

        self.grid[offset.y as usize][offset.x as usize]
    }

    /// Sets the value at the index, returning the previous entry
    /// if there was alread one there
    pub fn set(&mut self, index: IVec2, value: Entity) -> Option<Entity> {
        // if there are no entries, move the grid to the index
        if self.bound.is_empty() {
            debug!("empty");
            self.grid = vec![vec![Some(value)].into()].into();
            self.bound = IRect::from_corners(index, index + IVec2::ONE);
            return None;
        }

        // get the entry in the grid, grow to fit if needed
        let entry = loop {
            if index.y >= self.bound.max.y {
                let new_row = vec![None; self.grid[0].len()].into();
                self.grid.push_back(new_row);
                self.bound.max.y += 1;
                continue;
            }

            if index.y < self.bound.min.y {
                let new_row = vec![None; self.grid[0].len()].into();
                self.grid.push_front(new_row);
                self.bound.min.y -= 1;
                continue;
            }

            if index.x >= self.bound.max.x {
                for row in self.grid.iter_mut() {
                    row.push_back(None);
                }
                self.bound.max.x += 1;
                continue;
            }

            if index.x < self.bound.min.x {
                for row in self.grid.iter_mut() {
                    row.push_front(None);
                }
                self.bound.min.x -= 1;
                continue;
            }

            let offset = index - self.bound.min;

            break &mut self.grid[offset.y as usize][offset.x as usize];
        };

        std::mem::replace(entry, Some(value))
    }

    /// Removes an entry from the grid, returning it if it existed
    pub fn remove(&mut self, index: IVec2) -> Option<Entity> {
        if !self.contains(index) {
            return None;
        }

        let offset = index - self.bound.min;

        std::mem::replace(&mut self.grid[offset.y as usize][offset.x as usize], None)
    }

    /// shrinks the grid to fit the current set of entries
    pub fn trim(&mut self) {
        // remove empty rows from the positive y edge
        while let Some(row) = self.grid.back() {
            if row.iter().all(Option::is_none) {
                self.grid.pop_back();
                self.bound.max.y -= 1;
            } else {
                break;
            }
        }

        // remove empty rows from the negative y edge
        while let Some(row) = self.grid.front() {
            if row.iter().all(Option::is_none) {
                self.grid.pop_front();
                self.bound.min.y += 1;
            } else {
                break;
            }
        }

        // if all rows were removed reset bound
        if self.grid.is_empty() {
            self.bound = IRect::EMPTY;
            return;
        }

        // from here there is garunteed to be at least one entry

        // remove empty collumns from the positive x edge
        while self.grid.iter().all(|row| row.back().unwrap().is_none()) {
            for row in self.grid.iter_mut() {
                row.pop_back();
                self.bound.max.x -= 1;
            }
        }

        // remove empty collumns from the negative x edge
        while self.grid.iter().all(|row| row.front().unwrap().is_none()) {
            for row in self.grid.iter_mut() {
                row.pop_front();
                self.bound.min.x += 1;
            }
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct ShipModuleTransform {
    pub translation: IVec2,
    pub rotation: ModuleRotation,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ModuleRotation {
    /// No rotation.
    #[default]
    East,
    /// 90 degrees ccw.
    North,
    /// 90 degrees cw.
    West,
    /// 180 degrees.
    South,
}

impl ModuleRotation {
    pub fn next_cw(self) -> Self {
        use ModuleRotation::*;
        match self {
            East => South,
            North => East,
            West => North,
            South => West,
        }
    }

    pub fn next_ccw(self) -> Self {
        use ModuleRotation::*;
        match self {
            East => North,
            North => West,
            West => South,
            South => East,
        }
    }

    pub fn rotate_index(self, index: IVec2) -> IVec2 {
        use ModuleRotation::*;
        match self {
            East => IVec2::new(index.x, index.y),
            North => IVec2::new(-index.y, index.x),
            West => IVec2::new(-index.x, -index.y),
            South => IVec2::new(index.y, -index.x),
        }
    }
}

#[derive(Component)]
pub struct ShipModuleGridSpaces {
    pub spaces: Vec<IVec2>,
}

impl ShipModuleGridSpaces {
    /// Creates an iterator that returns the grid spaces transformed
    pub fn spaces_transformed(
        &self,
        transform: ShipModuleTransform,
    ) -> impl Iterator<Item = IVec2> + '_ {
        self.spaces
            .iter()
            .map(move |&space| transform.rotation.rotate_index(space) + transform.translation)
    }

    /// Returns `true` if there are no collisions with existing modules in a grid
    pub fn fits_in_grid(&self, transform: ShipModuleTransform, grid: &ShipModuleGrid) -> bool {
        self.spaces_transformed(transform)
            .all(|space| grid.get(space).is_none())
    }

    pub fn insert_into_grid(
        &self,
        transform: ShipModuleTransform,
        module_entity: Entity,
        grid: &mut ShipModuleGrid,
    ) {
        for space in self.spaces_transformed(transform) {
            if let Some(other_module_entity) = grid.set(space, module_entity) {
                error!(
                    "Placed module {} over other module {} at {}",
                    module_entity, other_module_entity, space
                );
            }
        }
    }
}

/// Shorthand for constructing [ShipModuleGridSpaces]
#[macro_export]
macro_rules! grid_spaces {
    [$(($x:expr, $y:expr)),* $(,)?] => {
        crate::modules::grid::ShipModuleGridSpaces {
            spaces: vec![$(IVec2::new($x, $y)),*],
        }
    };
}
