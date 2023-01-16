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
            x: x / self.cell_size,
            y: y / self.cell_size,
            z: z / self.cell_size,
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

    pub fn get_nearby(&self, position: Vec3, distance: f32) -> HashSet<Entity> {
        self.keys_to_query(position, distance)
        .into_iter()
        .filter_map(|key| self.cells.get(&key))
        .flat_map(|cell| cell.iter())
        .filter_map(|(entity, entity_position)| {
            if position.distance_squared(*entity_position) <= distance * distance {
                Some(*entity)
            } else {
                None
            }
        })
        .collect()
    }

    pub fn get_nearby_naive(&self, position: Vec3, distance: f32) -> HashSet<Entity> {
        self.cells.values()
        .flat_map(|cell| cell.iter())
        .filter_map(|(entity, entity_position)| {
            if position.distance_squared(*entity_position) <= distance * distance {
                Some(*entity)
            } else {
                None
            }
        })
        .collect()
    }

    fn keys_to_query(&self, position: Vec3, distance: f32) -> Vec<IVec3> {
        let position_key = self.key(position);
        let cell_distance = (distance / self.cell_size as f32).ceil() as i32;

        let mut keys = vec![];

        for x in (position_key.x - cell_distance)..(position_key.x + cell_distance) {
            for y in (position_key.y - cell_distance)..(position_key.y + cell_distance) {
                for z in (position_key.z - cell_distance)..(position_key.z + cell_distance) {
                    keys.push(IVec3::new(x, y, z));
                }
            }
        }

        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(grid.cells.get(&IVec3::new(1, 1, 1)).unwrap().get(&entity), Some(&position));
    }

    #[test]
    fn get_nearby_works() {
        let mut grid = SpatialHashGrid::new(5);
        let entity_positions = [
            (Entity::from_raw(0), Vec3::new(0.0, 0.0, 0.0)),
            (Entity::from_raw(1), Vec3::new(5.0, 5.0, 5.0)),
            (Entity::from_raw(2), Vec3::new(10.0, 10.0, 10.0)),
        ];

        for (entity, position) in entity_positions {
            grid.update(entity, position);
        }

        let nearby = grid.get_nearby(entity_positions[0].1, entity_positions[0].1.distance(entity_positions[1].1));
        assert!(nearby.contains(&entity_positions[0].0));
        assert!(nearby.contains(&entity_positions[1].0));

        let nearby = grid.get_nearby(entity_positions[1].1, entity_positions[1].1.distance(entity_positions[2].1));
        assert!(nearby.contains(&entity_positions[0].0));
        assert!(nearby.contains(&entity_positions[1].0));
        assert!(nearby.contains(&entity_positions[2].0));
    }
}