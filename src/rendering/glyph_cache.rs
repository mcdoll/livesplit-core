use super::{
    font::Font,
    mesh::{fill_builder, Mesh},
    Backend,
};
use hashbrown::HashMap;
use lyon::{
    path::{self, math::point, Path},
    tessellation::{FillOptions, FillTessellator},
};
use ttf_parser::OutlineBuilder;

struct PathBuilder(path::Builder);

impl OutlineBuilder for PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(point(x, -y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(point(x, -y));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quadratic_bezier_to(point(x1, -y1), point(x, -y));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0
            .cubic_bezier_to(point(x1, -y1), point(x2, -y2), point(x, -y));
    }
    fn close(&mut self) {
        self.0.close();
    }
}

pub struct GlyphCache<M> {
    glyphs: HashMap<u32, M>,
}

impl<M> Default for GlyphCache<M> {
    fn default() -> Self {
        Self {
            glyphs: Default::default(),
        }
    }
}

impl<M> GlyphCache<M> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn lookup_or_insert(
        &mut self,
        font: &Font<'_>,
        glyph: u32,
        backend: &mut impl Backend<Mesh = M>,
    ) -> &M {
        self.glyphs.entry(glyph).or_insert_with(|| {
            let mut builder = PathBuilder(Path::builder());
            let mut glyph_mesh = Mesh::new();

            if font.outline_glyph(glyph, &mut builder) {
                let path = builder.0.build();
                let mut tessellator = FillTessellator::new();
                tessellator
                    .tessellate_path(
                        &path,
                        &FillOptions::tolerance(0.005),
                        &mut fill_builder(&mut glyph_mesh),
                    )
                    .unwrap();

                let scale = -font.height().recip();
                let descender = scale * font.descender();

                for vertex in glyph_mesh.vertices_mut() {
                    vertex.v = descender + scale * vertex.y;
                }
            }

            backend.create_mesh(&glyph_mesh)
        })
    }
}
