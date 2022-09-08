use bevy::{prelude::*, utils::{HashMap, HashSet}};

pub struct SpatialHashGrid {
    cell_size: i32,
    cells: HashMap<IVec3, HashMap<Entity, Vec3>>,
}

impl SpatialHashGrid {
    pub fn new(cell_size: i32) -> Self {
        Self { cell_size, cells: Default::default() }
    }

    fn key(&self, position: Vec3) -> IVec3 {
        let x = position.x as i32;
        let y = position.y as i32;
        let z = position.z as i32;

        IVec3 {
            x: x - x % self.cell_size,
            y: y - y % self.cell_size,
            z: z - z % self.cell_size,
        }
    }

    pub fn update(&mut self, entity: Entity, position: Vec3) {
        let key = self.key(position);

        let cell = self.cells.entry(key).or_default();

        if cell.contains_key(&entity) {
            // entity is in same cell as before, simply update position
            cell.insert(entity, position);
        } else {
            // entity is in a new cell: remove entity from previous cell + insert into new cell
            // because an entity is only contained in one cell, we can break out of the loop early as soon as the entity was removed
            for cell in self.cells.values_mut() {
                if cell.remove(&entity).is_some() {
                    break;
                }
            }

            self.cells.entry(key).or_default().insert(entity, position);
        }
    }

    pub fn get_nearby(&self, position: Vec3, distance: u32) -> HashSet<Entity> {
        self.cells.values()
        .flat_map(|cell| cell.iter())
        .filter_map(|(entity, entity_position)| {
            if position.distance_squared(*entity_position) <= (distance * distance) as f32 {
                Some(*entity)
            } else {
                None
            }
        })
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_works() {
        let grid = SpatialHashGrid::new(5);

        assert_eq!(grid.key(Vec3::new(0.0, 0.0, 0.0)), IVec3::new(0, 0, 0));
        assert_eq!(grid.key(Vec3::new(4.5, 4.5, 4.5)), IVec3::new(0, 0, 0));
        assert_eq!(grid.key(Vec3::new(5.0, 5.0, 5.0)), IVec3::new(5, 5, 5));
        assert_eq!(grid.key(Vec3::new(-5.5, -5.5, -5.5)), IVec3::new(-5, -5, -5));
    }

    #[test]
    fn update_works() {
        let mut grid = SpatialHashGrid::new(5);
        let entity = Entity::from_raw(1);

        let position = Vec3::new(1.0, 1.0, 1.0);
        grid.update(entity, position);
        assert_eq!(grid.cells.get(&IVec3::new(0, 0, 0)).unwrap().get(&entity), Some(&position));

        let position = Vec3::new(5.5, 5.5, 5.5);
        grid.update(entity, position);
        assert_eq!(grid.cells.get(&IVec3::new(0, 0, 0)).unwrap().get(&entity), None);
        assert_eq!(grid.cells.get(&IVec3::new(5, 5, 5)).unwrap().get(&entity), Some(&position));
    }
}