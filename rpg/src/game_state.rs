use cgmath::Point2;
use hecs::World;
use toml::Table;
use bananagraph::{DrawingContext, GpuWrapper, IdBuffer, WindowEventHandler};
use crate::components::OnMap;
use crate::map_file::load_toml;

pub struct GameState {
    world: World
}

impl GameState {
    pub fn new(_seed: u64) -> Self {
        Self { world: World::new() }
    }
}

impl WindowEventHandler for GameState {
    fn init(&mut self, wrapper: &mut GpuWrapper) {
        // The order of these is important, it maps to Layers impling Into<u32> in sprites
        wrapper.add_texture(include_bytes!("art/Dungeon.png"), Some("Dungeon.png"));

        // Just load a quick map
        const test_map: &str = r#"
            [map]
            cells = """
                ################
                #l.b.#.#l.Dcc..#
                #....#.+.....A.#
                #C...+.#.......#
                ######.#########
                #l...p.p......l#
                #..............#
                ######+#########
            """
            "#;
        let table = test_map.parse::<Table>().unwrap();
        load_toml(&mut self.world, table).unwrap();
    }

    fn redraw(&self, _mouse_pos: Point2<f64>, wrapper: &GpuWrapper) -> Option<IdBuffer> {
        let zoom = 2.0;

        let dc = DrawingContext::new((wrapper.logical_size.x as f32 / zoom, wrapper.logical_size.y as f32 / zoom));
        let mut sprites = vec![];
        sprites.append(&mut OnMap::system(&self.world, dc));
        wrapper.redraw(sprites);
        None
    }
}