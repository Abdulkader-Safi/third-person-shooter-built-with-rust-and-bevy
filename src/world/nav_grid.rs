use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

pub struct NavGridPlugin;

impl Plugin for NavGridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NavGrid::new(100, 100, 1.0));
    }
}

/// Navigation grid for A* pathfinding
#[derive(Resource)]
pub struct NavGrid {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    grid: Vec<bool>, // true = walkable
    offset: Vec2,    // World offset (grid center at world origin)
}

impl NavGrid {
    pub fn new(width: usize, height: usize, cell_size: f32) -> Self {
        let grid = vec![true; width * height];
        let offset = Vec2::new(
            -(width as f32 * cell_size) / 2.0,
            -(height as f32 * cell_size) / 2.0,
        );
        Self {
            width,
            height,
            cell_size,
            grid,
            offset,
        }
    }

    /// Convert world position to grid coordinates
    pub fn world_to_grid(&self, pos: Vec3) -> Option<(usize, usize)> {
        let x = ((pos.x - self.offset.x) / self.cell_size) as i32;
        let y = ((pos.z - self.offset.y) / self.cell_size) as i32;

        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            Some((x as usize, y as usize))
        } else {
            None
        }
    }

    /// Convert grid coordinates to world position (center of cell)
    pub fn grid_to_world(&self, x: usize, y: usize) -> Vec3 {
        Vec3::new(
            self.offset.x + (x as f32 + 0.5) * self.cell_size,
            0.0,
            self.offset.y + (y as f32 + 0.5) * self.cell_size,
        )
    }

    /// Check if a cell is walkable
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.grid[y * self.width + x]
        } else {
            false
        }
    }

    /// Mark a cell as an obstacle (not walkable)
    pub fn set_obstacle(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.grid[y * self.width + x] = false;
        }
    }

    /// Mark a rectangular area as obstacles
    pub fn set_obstacle_rect(&mut self, min_x: usize, min_y: usize, max_x: usize, max_y: usize) {
        for y in min_y..=max_y.min(self.height - 1) {
            for x in min_x..=max_x.min(self.width - 1) {
                self.grid[y * self.width + x] = false;
            }
        }
    }

    /// Mark obstacles from world coordinates and size
    pub fn mark_obstacle_world(&mut self, pos: Vec3, half_extents: Vec3) {
        let min_world = pos - Vec3::new(half_extents.x, 0.0, half_extents.z);
        let max_world = pos + Vec3::new(half_extents.x, 0.0, half_extents.z);

        if let (Some((min_x, min_y)), Some((max_x, max_y))) =
            (self.world_to_grid(min_world), self.world_to_grid(max_world))
        {
            self.set_obstacle_rect(min_x, min_y, max_x, max_y);
        }
    }

    /// Find path using A* algorithm
    pub fn find_path(&self, start: Vec3, end: Vec3) -> Option<Vec<Vec3>> {
        let start_node = self.world_to_grid(start)?;
        let end_node = self.world_to_grid(end)?;

        // If start or end is not walkable, return None
        if !self.is_walkable(start_node.0, start_node.1) {
            return None;
        }

        // If end is not walkable, find nearest walkable cell
        let end_node = if !self.is_walkable(end_node.0, end_node.1) {
            self.find_nearest_walkable(end_node)?
        } else {
            end_node
        };

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut g_score: HashMap<(usize, usize), f32> = HashMap::new();
        let mut closed_set: HashSet<(usize, usize)> = HashSet::new();

        g_score.insert(start_node, 0.0);
        open_set.push(Node {
            pos: start_node,
            f_score: self.heuristic(start_node, end_node),
        });

        while let Some(current) = open_set.pop() {
            if current.pos == end_node {
                return Some(self.reconstruct_path(&came_from, current.pos));
            }

            if closed_set.contains(&current.pos) {
                continue;
            }
            closed_set.insert(current.pos);

            // Check 8 neighbors (including diagonals)
            for neighbor in self.get_neighbors(current.pos) {
                if closed_set.contains(&neighbor) {
                    continue;
                }

                let move_cost = if neighbor.0 != current.pos.0 && neighbor.1 != current.pos.1 {
                    1.414 // Diagonal movement
                } else {
                    1.0 // Cardinal movement
                };

                let tentative_g = g_score.get(&current.pos).unwrap_or(&f32::MAX) + move_cost;

                if tentative_g < *g_score.get(&neighbor).unwrap_or(&f32::MAX) {
                    came_from.insert(neighbor, current.pos);
                    g_score.insert(neighbor, tentative_g);
                    let f = tentative_g + self.heuristic(neighbor, end_node);
                    open_set.push(Node {
                        pos: neighbor,
                        f_score: f,
                    });
                }
            }
        }

        None // No path found
    }

    fn heuristic(&self, a: (usize, usize), b: (usize, usize)) -> f32 {
        // Euclidean distance
        let dx = (a.0 as f32 - b.0 as f32).abs();
        let dy = (a.1 as f32 - b.1 as f32).abs();
        (dx * dx + dy * dy).sqrt()
    }

    fn get_neighbors(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::with_capacity(8);
        let (x, y) = pos;

        // 8 directions: N, S, E, W, NE, NW, SE, SW
        let directions: [(i32, i32); 8] = [
            (0, 1),
            (0, -1),
            (1, 0),
            (-1, 0),
            (1, 1),
            (-1, 1),
            (1, -1),
            (-1, -1),
        ];

        for (dx, dy) in directions {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                let nx = nx as usize;
                let ny = ny as usize;

                if self.is_walkable(nx, ny) {
                    // For diagonal movement, check that we can actually move diagonally
                    // (not cutting corners through walls)
                    if dx != 0 && dy != 0 {
                        let can_move_x = self.is_walkable((x as i32 + dx) as usize, y);
                        let can_move_y = self.is_walkable(x, (y as i32 + dy) as usize);
                        if can_move_x && can_move_y {
                            neighbors.push((nx, ny));
                        }
                    } else {
                        neighbors.push((nx, ny));
                    }
                }
            }
        }

        neighbors
    }

    fn reconstruct_path(
        &self,
        came_from: &HashMap<(usize, usize), (usize, usize)>,
        mut current: (usize, usize),
    ) -> Vec<Vec3> {
        let mut path = vec![self.grid_to_world(current.0, current.1)];

        while let Some(&prev) = came_from.get(&current) {
            current = prev;
            path.push(self.grid_to_world(current.0, current.1));
        }

        path.reverse();

        // Simplify path by removing intermediate points on straight lines
        self.simplify_path(path)
    }

    fn simplify_path(&self, path: Vec<Vec3>) -> Vec<Vec3> {
        if path.len() <= 2 {
            return path;
        }

        let mut simplified = vec![path[0]];
        let mut i = 0;

        while i < path.len() - 1 {
            let mut j = path.len() - 1;

            // Find the furthest point we can reach in a straight line
            while j > i + 1 {
                if self.can_walk_straight(path[i], path[j]) {
                    break;
                }
                j -= 1;
            }

            simplified.push(path[j]);
            i = j;
        }

        simplified
    }

    fn can_walk_straight(&self, start: Vec3, end: Vec3) -> bool {
        let dist = (end - start).length();
        let steps = (dist / (self.cell_size * 0.5)) as i32;

        if steps <= 1 {
            return true;
        }

        for i in 1..steps {
            let t = i as f32 / steps as f32;
            let pos = start.lerp(end, t);

            if let Some((x, y)) = self.world_to_grid(pos) {
                if !self.is_walkable(x, y) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn find_nearest_walkable(&self, pos: (usize, usize)) -> Option<(usize, usize)> {
        // Search in expanding squares around the target
        for radius in 1..10 {
            for dx in -(radius as i32)..=(radius as i32) {
                for dy in -(radius as i32)..=(radius as i32) {
                    if dx.abs() != radius as i32 && dy.abs() != radius as i32 {
                        continue; // Only check perimeter
                    }

                    let nx = pos.0 as i32 + dx;
                    let ny = pos.1 as i32 + dy;

                    if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                        let nx = nx as usize;
                        let ny = ny as usize;
                        if self.is_walkable(nx, ny) {
                            return Some((nx, ny));
                        }
                    }
                }
            }
        }
        None
    }
}

/// Node for A* priority queue
#[derive(Clone)]
struct Node {
    pos: (usize, usize),
    f_score: f32,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other
            .f_score
            .partial_cmp(&self.f_score)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
