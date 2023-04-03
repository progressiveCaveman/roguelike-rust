// find an item and try to pick it up
for tile in vs.visible_tiles.iter() {
    let idx = map.xy_idx(tile.x, tile.y);
    let entities = &map.tile_content[idx];
    for e in entities.iter() {
        if let Ok(name) = world.get::<Name>(*e){
            dbg!(&name.name);
        }

        if let Ok(item) = world.get::<Item>(*e){
            if let Ok(p) = world.get::<Position>(*e){
                println!("Found an item");

                // visible_items.push(*e);
                let p = p.ps[0];

                dbg!(p);
                dbg!(pos.ps[0]);

                if p == pos.ps[0] {
                    println!("Needs want pickup");
                    //add wants to pick up intent and return
                    needs_wants_to_pick_up.push((id, *e));
                    break;
                } else {
                    retargeted = true;
                    target = p.clone();
                }
            }
        }

        // match world.get::<(Item, Position)>(*e) {
        //     Err(_e) => {},
        //     Ok(awe) => {

        //     }
        // }
    }
}

// implementing generics
pub struct Screen<T: Draw> {
    pub components: Vec<T>,
}

impl<T> Screen<T>
where
    T: Draw,
{
    pub fn run(&self) {
        for component in self.components.iter() {
            component.draw();
        }
    }
}

/// Storing a closure as a struct
/// https://stackoverflow.com/questions/27831944/how-do-i-store-a-closure-in-a-struct-in-rust
// Unboxed closure
struct Foo<F>
where
    F: Fn(usize) -> usize,
{
    pub foo: F,
}

impl<F> Foo<F>
where
    F: Fn(usize) -> usize,
{
    fn new(foo: F) -> Self {
        Self { foo }
    }
}

fn main() {
    let foo = Foo { foo: |a| a + 1 };
    (foo.foo)(42);
    
    (Foo::new(|a| a + 1).foo)(42);
}

// Boxed trait object
struct Foo {
    pub foo: Box<dyn Fn(usize) -> usize>,
}

impl Foo {
    fn new(foo: impl Fn(usize) -> usize + 'static) -> Self {
        Self { foo: Box::new(foo) }
    }
}

fn main() {
    let foo = Foo {
        foo: Box::new(|a| a + 1),
    };
    (foo.foo)(42);
    
    (Foo::new(|a| a + 1).foo)(42);
}

// Trait object reference
struct Foo<'a> {
    pub foo: &'a dyn Fn(usize) -> usize,
}

impl<'a> Foo<'a> {
    fn new(foo: &'a dyn Fn(usize) -> usize) -> Self {
        Self { foo }
    }
}

fn main() {
    let foo = Foo { foo: &|a| a + 1 };
    (foo.foo)(42);
    
    (Foo::new(&|a| a + 1).foo)(42);
}

// Function pointer
struct Foo {
    pub foo: fn(usize) -> usize,
}

impl Foo {
    fn new(foo: fn(usize) -> usize) -> Self {
        Self { foo }
    }
}

fn main() {
    let foo = Foo { foo: |a| a + 1 };
    (foo.foo)(42);
    
    (Foo::new(|a| a + 1).foo)(42);
}
