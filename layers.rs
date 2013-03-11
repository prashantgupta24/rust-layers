use geom::matrix::{Matrix4, identity};
use geom::size::Size2D;
use opengles::gl2::{GLuint, delete_textures};

use std::cmp::FuzzyEq;

pub enum Format {
    ARGB32Format,
    RGB24Format
}

pub enum Layer {
    ContainerLayerKind(@mut ContainerLayer),
    ImageLayerKind(@mut ImageLayer),
    TiledImageLayerKind(@mut TiledImageLayer)
}

impl Layer {
    pure fn with_common<T>(&self, f: &fn(&mut CommonLayer) -> T) -> T {
        match *self {
            ContainerLayerKind(container_layer) => f(&mut container_layer.common),
            ImageLayerKind(image_layer) => f(&mut image_layer.common),
            TiledImageLayerKind(tiled_image_layer) => f(&mut tiled_image_layer.common)
        }
    }
}

pub struct CommonLayer {
    parent: Option<Layer>,
    prev_sibling: Option<Layer>,
    next_sibling: Option<Layer>,

    transform: Matrix4<f32>,
}

pub impl CommonLayer {
    // FIXME: Workaround for cross-crate bug regarding mutability of class fields
    fn set_transform(&mut self, new_transform: Matrix4<f32>) {
        self.transform = new_transform;
    }
}

pub fn CommonLayer() -> CommonLayer {
    CommonLayer {
        parent: None,
        prev_sibling: None,
        next_sibling: None,
        transform: identity(),
    }
}


pub struct ContainerLayer {
    common: CommonLayer,
    first_child: Option<Layer>,
    last_child: Option<Layer>,
}


pub fn ContainerLayer() -> ContainerLayer {
    ContainerLayer {
        common: CommonLayer(),
        first_child: None,
        last_child: None,
    }
}

pub impl ContainerLayer {
    fn each_child(&const self, f: &fn(Layer) -> bool) {
        let mut child_opt = self.first_child;
        while !child_opt.is_none() {
            let child = child_opt.get();
            if !f(child) { break; }
            child_opt = child.with_common(|x| x.next_sibling);
        }
    }

    /// Only works when the child is disconnected from the layer tree.
    fn add_child(&mut self, new_child: Layer) {
        do new_child.with_common |new_child_common| {
            assert new_child_common.parent.is_none();
            assert new_child_common.prev_sibling.is_none();
            assert new_child_common.next_sibling.is_none();

            match self.first_child {
                None => {}
                Some(copy first_child) => {
                    do first_child.with_common |first_child_common| {
                        assert first_child_common.prev_sibling.is_none();
                        first_child_common.prev_sibling = Some(new_child);
                        new_child_common.next_sibling = Some(first_child);
                    }
                }
            }

            self.first_child = Some(new_child);

            match self.last_child {
                None => self.last_child = Some(new_child),
                Some(_) => {}
            }
        }
    }
}

pub type WithDataFn = &'self fn(&'self [u8]);

pub trait ImageData {
    fn size(&self) -> Size2D<uint>;

    // NB: stride is in pixels, like OpenGL GL_UNPACK_ROW_LENGTH.
    fn stride(&self) -> uint;

    fn format(&self) -> Format;
    fn with_data(&self, WithDataFn);
}

pub struct Image {
    data: @mut ImageData,
    texture: Option<GLuint>,

    drop {
        match copy self.texture {
            None => {
                // Nothing to do.
            }
            Some(texture) => {
                delete_textures(&[texture]);
            }
        }
    }
}

pub impl Image {
    static fn new(data: @mut ImageData) -> Image {
        Image { data: data, texture: None }
    }
}

/// Basic image data is a simple image data store that just owns the pixel data in memory.
pub struct BasicImageData {
    size: Size2D<uint>,
    stride: uint,
    format: Format,
    data: ~[u8]
}

pub impl BasicImageData {
    static fn new(size: Size2D<uint>, stride: uint, format: Format, data: ~[u8]) ->
            BasicImageData {
        BasicImageData {
            size: size,
            stride: stride,
            format: format,
            data: data
        }
    }
}

impl ImageData for BasicImageData {
    fn size(&self) -> Size2D<uint> { self.size }
    fn stride(&self) -> uint { self.stride }
    fn format(&self) -> Format { self.format }
    fn with_data(&self, f: WithDataFn) { f(self.data) }
}

pub struct ImageLayer {
    common: CommonLayer,
    image: @mut Image,
}

pub impl ImageLayer {
    // FIXME: Workaround for cross-crate bug
    fn set_image(&mut self, new_image: @mut Image) {
        self.image = new_image;
    }
}

pub fn ImageLayer(image: @mut Image) -> ImageLayer {
    ImageLayer {
        common : CommonLayer(),
        image : image,
    }
}

pub struct TiledImageLayer {
    common: CommonLayer,
    tiles: @mut ~[@mut Image],
    tiles_across: uint,
}

pub fn TiledImageLayer(in_tiles: &[@mut Image], tiles_across: uint) -> TiledImageLayer {
    let mut tiles = @mut ~[];
    for in_tiles.each |tile| {
        tiles.push(*tile);
    }

    TiledImageLayer {
        common: CommonLayer(),
        tiles: tiles,
        tiles_across: tiles_across
    }
}

