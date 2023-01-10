/*******************************************

    Using Godot_v3.4.5-stable_win64

*******************************************/

use std::ops;
use std::vec;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::cmp::Ordering;
use core::slice::IterMut;
use core::iter::Enumerate;

use gdnative::prelude::*;


/***************************

    Containers

**************************/

// Table

#[derive(Default)]
struct IndexTable {
    head: u16,
    list: Vec<u16>,
}

impl IndexTable {
    const END_OF_LIST: u16 = 0;

    fn new() -> Self {
        Self {
            head: IndexTable::END_OF_LIST,
            list: vec![IndexTable::END_OF_LIST], 
        }
    }

    fn reset(&mut self) {
        self.head = IndexTable::END_OF_LIST;
        self.list.clear();
        self.list.push(IndexTable::END_OF_LIST);
    }

    fn allocate(&mut self) -> u16 {
        let mut value = self.head;
        if value == IndexTable::END_OF_LIST {
            value = self.list.len().try_into().unwrap();
            self.list.push(IndexTable::END_OF_LIST);
        } else {
            let index: usize = value.into();
            self.head = self.list[index];
            self.list[index] = IndexTable::END_OF_LIST;
        }
        //godot_print!("allocate {}", value);
        value
    }

    fn free(&mut self, value: u16) {
        //godot_print!("free {}", value);
        let index: usize = value.into();
        if self.list[index] == IndexTable::END_OF_LIST {
            self.list[index] = self.head;
            self.head = value;
        }
    }
}




/***************************

    MersenneTwister

**************************/

#[derive(Copy, Clone)]
struct MersenneTwister {
    p: i32,
    q: i32,
    r: i32,
    x: [u32; MersenneTwister::N],
}

impl MersenneTwister {
     // static data and functions
    const N: usize = 624;
    const M: i32 = 397;
    const UPPER_MASK: u32 = 0x80000000;
    const LOWER_MASK: u32 = 0x7fffffff;
    const MATRIX_A: u32 = 0x9908b0df;

    fn new() -> Self {
        Self {
            p: 0,
            q: 0,
            r: 0,
            x: [0; MersenneTwister::N],
        }
    }
    
    fn from_zero(s: u32) -> Self {
        let mut r = MersenneTwister::new();
        r.set_seed(0);
        r
    }

    fn from(s: u32) -> Self {
        let mut r = MersenneTwister::new();
        r.set_seed(s);
        r
    }

    fn imul(a: u32, b: u32) -> u32 {
        let al: u32 = a & 0xffff;
        let ah: u32 = a >> 16;
        let bl: u32 = b & 0xffff;
        let bh: u32 = b >> 16;
        let ml: u32 = al * bl;
        let mh: u32 = ( (((ml >> 16) + al * bh) & 0xffff) + ah * bl ) & 0xffff;
        (mh << 16) | (ml & 0xffff)
    }

    fn set_seed(&mut self, s: u32) {
        self.x[0] = s;
        for i in 1..MersenneTwister::N {
            let i_u32: u32 = i.try_into().unwrap();
            self.x[i] = MersenneTwister::imul( 1812433253, self.x[i - 1] ^ (self.x[i - 1] >> 30) ) + i_u32;
            self.x[i] &= 0xffffffff;
        }
        self.p = 0;
        self.q = 1;
        self.r = MersenneTwister::M;
    }

    fn next_u32(&mut self) -> u32 {
        let p: usize = self.p.try_into().unwrap();
        let q: usize = self.q.try_into().unwrap();
        let r: usize = self.r.try_into().unwrap();

        let mut y: u32 = (self.x[p] & MersenneTwister::UPPER_MASK) | (self.x[q] & MersenneTwister::LOWER_MASK);
        self.x[p] = self.x[r] ^ (y >> 1) ^ ((y & 1) * MersenneTwister::MATRIX_A);
        y = self.x[p];

        self.p += 1;
        self.q += 1;
        self.r += 1;

        let N: i32 = MersenneTwister::N.try_into().unwrap();
        if self.p == N { self.p = 0; }
        if self.q == N { self.q = 0; }
        if self.r == N { self.r = 0; }

        y ^= (y >> 11);
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= (y >> 18);

        y
    }

    fn next_u32_bits(&mut self, bits: u32) -> u32 {
        self.next_u32() >> (32 - bits)
    }

    fn next_from_zero(&mut self, max: i32) -> i32 {        
        let n = self.next_u32() % u32::try_from(max + 1).unwrap();
        i32::try_from(n).unwrap()
    }

    fn next_range(&mut self, min: i32, max: i32) -> i32 {        
        let n = self.next_u32() % u32::try_from(max + 1 - min).unwrap();
        i32::try_from(n).unwrap() + min
    }

}


/***************************

    Vector2

**************************/

#[derive(Default, Copy, Clone)]
struct Vector2 {
    x: i32,
    y: i32,
}

impl ops::Neg for Vector2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
    }
}

impl ops::Add for Vector2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y }
    }
}

impl ops::Sub for Vector2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {x: self.x - other.x, y: self.y - other.y}
    }
}

impl ops::AddAssign for Vector2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl ops::SubAssign for Vector2 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Vector2 {
    fn new() -> Self {
        Self { x: 0, y: 0 }
    }

    fn from(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}


/***************************

    Entity

**************************/
#[derive(Default, Copy, Clone)]
struct Entity {
    index: u16,
    generation: u16,
}

impl Entity {
    fn new() -> Self {
        Self { index: 0, generation: 0 }
    }

    fn from_index(i: usize) -> Self {
        Self { index: i.try_into().unwrap(), generation: 0 }
    }

    // do i need this anymore?
    fn from_unknown(i: u16) -> Self {
        Self { index: 0, generation: 0 }
    }

    fn from(index: u16, generation: u16) -> Self {
        Self { index, generation }
    }

    fn is_null(&self) -> bool {
        //self.index == 0 && self.generation == 0
        self.index == 0
    }

    fn id(&self) -> u16 {
        self.index
    }

    fn generation(&self) -> u16 {
        self.generation
    }

    fn index(&self) -> usize {
        self.index.into()
    }
}





/***************************

    Components

**************************/
#[derive(Default, Copy, Clone)]
struct Animator {
    frame: u16,
    count: u16,
}

#[derive(Default, Copy, Clone)]
struct Body {
    position: Vector2,
    velocity: Vector2,
    size: Vector2,
}

#[derive(Default, Copy, Clone)]
struct Player {
    slot: i8,
    delayFire: u16,
    damage: u16,
}

#[derive(Default, Copy, Clone)]
struct Enemy {
    direction: i8,
    counter: u8,
    delayFire: u16,
}

#[derive(Default, Copy, Clone)]
struct ObjType {
    value: u8
}

impl ObjType {
    const Null: u8 = 0;
    const Player: u8 = 1;
    const Enemy: u8 = 2;
    const Bullet: u8 = 3;
    const BadBullet: u8 = 4;
    const Boom: u8 = 5;
    const PlayerBoom: u8 = 6;
    const ShotCleaner: u8 = 7;
    const Count: u8 = 8;
    const USizeCount:usize = 8;

    fn from(value: u8) -> ObjType {
        Self { value }
    }
}

#[derive(Default, Copy, Clone)]
struct Cf {
    value: u8
}

impl Cf {

    fn from(value: u8) -> Cf {
        Cf { value }
    }

    fn test(&self, mask: u8) -> bool {
        self.value & mask == mask
    }

    fn none(&self) -> bool {
        self.value == Cf::None
    }
}


impl Cf {
    const None: u8 = 0;
    const Component: u8 = 1 << 0;
    const ObjectId: u8 = 1 << 1;
    const Body: u8 = 1 << 2;
    const Player: u8 = 1 << 3;
    const Enemy: u8 = 1 << 4;
    const Animator: u8 = 1 << 5;
    const Active: u8 = 1 << 6;
}

/***************************

    Utility Objects

**************************/
#[derive(Default, Copy, Clone)]
struct BitMask {
    mask:i64,
    shift:usize,
}

impl BitMask {
    fn new() -> Self {
        Self {
            mask: 0,
            shift: 0,
        }
    }
    
    const fn const_from(start:usize, end:usize) -> Self {
        let mut ender = end;
        if ender >= 64 { ender = 63; }

        let mut mask = 0x0000000000000000;
        let mut i = start;

        while i < ender {
            mask |= 1 << i;
            i += 1;
        }

        Self {
            mask: mask,
            shift: start,
        }
    }


    fn from(start:usize, end:usize) -> Self {
        let mut ender = end;
        if ender >= 64 { ender = 63; }

        let mut mask = 0x0000000000000000;
        let shift = start;
        for i in start..ender { 
            mask |= 1 << i;
        }

        Self {
            mask: mask,
            shift: shift,
        }
    }

    fn decode(&self, value:i64) -> i64 {
        (value & self.mask) >> self.shift
    }

    fn encode(&self, value:i64) -> i64 {
        (value << self.shift) & self.mask
    }
}


#[derive(Default, Copy, Clone)]
struct Control {
    // 3 bits state - 0 is unready, 1+ difficulty, 7 synced
    // non empty 1 bit
    // left, right, primary 3 bits
    state: u8,
    x: i16,

    // 10 bits, 6 left over
    nonEmpty: bool,
    left: bool,
    right: bool,

    primary: bool,

    debug: u32,
}
    
// maybe combine this with above
#[derive(Default, Copy, Clone)]
struct Slot {
    connected: bool,
    connectedAtStart: bool,
    broken: bool,
    raw: i64,
    input: Control,
}

#[derive(Default, Copy, Clone)]
struct GlobalState {
    playing: bool,
    enemySpeed: i32,
    enemyCount: i32,
    textType: u16,
    textAnimate: i32,
}

#[derive(Default, Copy, Clone)]
struct Bounds {
    btype: u8,
    entity: Entity,
    lower: Vector2,
    upper: Vector2,
}
/*
impl PartialOrd for Bounds {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.lower.x < other.lower.x
    }
}

impl PartialEq for Bounds {
    fn eq(&self, other: &Self) -> Option<Ordering> {
        self.lower.x == other.lower.x
    }
}
*/
impl Bounds {

    fn from(entity:Entity, btype:u8, p:Vector2, d:Vector2, s:Vector2) -> Self {
        Self {
            btype,
            entity,
            lower: p - s,
            upper: p + s,
        }
    }

    fn less_than(&self, other: &Self) -> bool {
        self.lower.x < other.lower.x
    }

    fn overlap(&self, b: &Self) -> bool {
        if b.lower.x > self.upper.x || b.upper.x < self.lower.x ||
            b.lower.y > self.upper.y || b.upper.y < self.lower.y {
            return false
        }
        true
    }
}

#[derive(Default, Copy, Clone)]
struct Event {
    id: u8,
    a: Entity,
    b: Entity,
    key: u16,
    otype: u8,
    v: Vector2,
}

impl Event {
    const Null: u8 = 0;
    const DestroyEntity: u8 = 1;
    const CreateEntity: u8 = 2;
    const Contact: u8 = 3;
    const Shoot: u8 = 4;
    const Count: u8 = 5;

    fn from_destroy(a:Entity) -> Self {
        Self {
            id: Event::DestroyEntity,
            a,
            b: Entity::new(),
            key: 0,
            otype: ObjType::Null,
            v: Vector2::new(),
        }
    }

    fn from_contact(key:u16, a:Entity, b:Entity) -> Self {
        Self {
            id: Event::Contact,
            a,
            b,
            key,
            otype: ObjType::Null,
            v: Vector2::new(),
        }
    }

    fn from_entity(otype: u8, v:Vector2) -> Self {
        Self {
            id: Event::CreateEntity,
            a: Entity::new(),
            b: Entity::new(),
            key: 0,
            otype,
            v,
        }
    }

    fn from_player(key: u16, v:Vector2) -> Self {
        Self {
            id: Event::CreateEntity,
            a: Entity::new(),
            b: Entity::new(),
            key,
            otype: ObjType::Player,
            v,
        }
    }

}

/***************************

    Component Pools

**************************/

/*
    for item in 0..5 {
        println!("{}", item);
    }

    let mut iterator = (0..5).into_iter();
    while let Some(item) = iterator.next() {
        println!("{}", item);
    }
*/

#[derive(Default)]
struct CpPack {
    generation: Vec<u16>,
    comp: Vec<Cf>,
    objectId: Vec<ObjType>,
    body: Vec<Body>,
    player: Vec<Player>,
    enemy: Vec<Enemy>,
    animator: Vec<Animator>,
}

impl CpPack {

    fn new() -> CpPack { 
        Default::default()
    }

    fn iter(&mut self) -> CpIterMut {
        CpIterMut {
            mask: 0,
            generation: self.generation.iter_mut().enumerate(),
            comp: self.comp.iter_mut().enumerate(),
            objectId: self.objectId.iter_mut().enumerate(),
            body: self.body.iter_mut().enumerate(),
            player: self.player.iter_mut().enumerate(),
            enemy: self.enemy.iter_mut().enumerate(),
            animator: self.animator.iter_mut().enumerate(),
        }
    }

    fn filter(&mut self, mask: u8) -> CpIterMut {
        CpIterMut {
            mask: mask,
            generation: self.generation.iter_mut().enumerate(),
            comp: self.comp.iter_mut().enumerate(),
            objectId: self.objectId.iter_mut().enumerate(),
            body: self.body.iter_mut().enumerate(),
            player: self.player.iter_mut().enumerate(),
            enemy: self.enemy.iter_mut().enumerate(),
            animator: self.animator.iter_mut().enumerate(),
        }
    }

    fn clear(&mut self) {
        self.generation.clear();
        self.comp.clear();
        self.objectId.clear();
        self.body.clear();
        self.player.clear();
        self.enemy.clear();
        self.animator.clear();
    }

    fn size(&self) -> usize {
        self.generation.len()
    }

    fn resize(&mut self, s: usize) {
        if self.size() < s {
            self.generation.resize(s, Default::default());
            self.comp.resize(s, Default::default());
            self.objectId.resize(s, Default::default());
            self.body.resize(s, Default::default());
            self.player.resize(s, Default::default());
            self.enemy.resize(s, Default::default());
            self.animator.resize(s, Default::default());
        }
    }

    fn smartCopy(&mut self, other: &CpPack) {
        self.generation.resize(other.generation.len(), Default::default());
        self.comp.resize(other.comp.len(), Default::default());
        self.objectId.resize(other.objectId.len(), Default::default());
        self.body.resize(other.body.len(), Default::default());
        self.player.resize(other.player.len(), Default::default());
        self.enemy.resize(other.enemy.len(), Default::default());
        self.animator.resize(other.animator.len(), Default::default());

        for i in 0..other.generation.len() { self.generation[i] = other.generation[i]; }
        for i in 0..other.comp.len() { self.comp[i] = other.comp[i]; }
        for i in 0..other.objectId.len() { self.objectId[i] = other.objectId[i]; }
        for i in 0..other.body.len() { self.body[i] = other.body[i]; }
        for i in 0..other.player.len() { self.player[i] = other.player[i]; }
        for i in 0..other.enemy.len() { self.enemy[i] = other.enemy[i]; }
        for i in 0..other.animator.len() { self.animator[i] = other.animator[i]; }
    }

}

struct CpReference<'a> {
    entity: Entity,
    comp: &'a mut Cf,
    objectId: &'a mut ObjType,
    body: &'a mut Body,
    player: &'a mut Player,
    enemy: &'a mut Enemy,
    animator: &'a mut Animator,
}

struct CpIterMut<'a> {
    mask: u8,
    generation: Enumerate<IterMut<'a, u16>>,
    comp: Enumerate<IterMut<'a, Cf>>,
    objectId: Enumerate<IterMut<'a, ObjType>>,
    body: Enumerate<IterMut<'a, Body>>,
    player: Enumerate<IterMut<'a, Player>>,
    enemy: Enumerate<IterMut<'a, Enemy>>,
    animator: Enumerate<IterMut<'a, Animator>>,
}

impl<'a> CpIterMut<'a> {

    fn nth(&mut self, e: Entity) -> Option<CpReference> {
        let n = e.index();
        let generation = self.generation.nth(n);
        let comp = self.comp.nth(n);
        let objectId = self.objectId.nth(n);
        let body = self.body.nth(n);
        let player = self.player.nth(n);
        let enemy = self.enemy.nth(n);
        let animator = self.animator.nth(n);
        if generation == None {
            None
        } else {
            let g = generation.unwrap();
            Some(
                CpReference {
                    entity: Entity::from(g.0.try_into().unwrap(), *g.1),
                    comp: comp.unwrap().1,
                    objectId: objectId.unwrap().1,
                    body: body.unwrap().1,
                    player: player.unwrap().1,
                    enemy: enemy.unwrap().1,
                    animator: animator.unwrap().1,
                }
            )
        }
    }

    fn nth_double(&mut self, a: Entity, b: Entity) -> Option<(CpReference, CpReference)> {
    
        let ia = a.index();
        let ib = b.index();

        if ia == ib {
            return None;
        }

        let swapped = ia > ib;

        let mut n0 = ia;
        let mut n1 = ib;

        if swapped == true {
            n0 = ib;
            n1 = ia;
        }

        n1 = n1 - (n0 + 1);

        let mut A: CpReference;
        let mut B: CpReference;

        let generation = self.generation.nth(n0);
        let comp = self.comp.nth(n0);
        let objectId = self.objectId.nth(n0);
        let body = self.body.nth(n0);
        let player = self.player.nth(n0);
        let enemy = self.enemy.nth(n0);
        let animator = self.animator.nth(n0);
        
        if generation == None {
            return None;
        } else {
            let g = generation.unwrap();
            A = CpReference {
                entity: Entity::from(g.0.try_into().unwrap(), *g.1),
                comp: comp.unwrap().1,
                objectId: objectId.unwrap().1,
                body: body.unwrap().1,
                player: player.unwrap().1,
                enemy: enemy.unwrap().1,
                animator: animator.unwrap().1,
            };
        }

        let generation = self.generation.nth(n1);
        let comp = self.comp.nth(n1);
        let objectId = self.objectId.nth(n1);
        let body = self.body.nth(n1);
        let player = self.player.nth(n1);
        let enemy = self.enemy.nth(n1);
        let animator = self.animator.nth(n1);
        
        if generation == None {
            return None;
        } else {
            let g = generation.unwrap();
            B = CpReference {
                entity: Entity::from(g.0.try_into().unwrap(), *g.1),
                comp: comp.unwrap().1,
                objectId: objectId.unwrap().1,
                body: body.unwrap().1,
                player: player.unwrap().1,
                enemy: enemy.unwrap().1,
                animator: animator.unwrap().1,
            };
        }

        if swapped == true {
            return Some((B, A));
        } else {
            return Some((A, B));
        }

    }

}

impl<'a> Iterator for CpIterMut<'a> {
    type Item = CpReference<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut done = false;
        while done == false {
            let generation = self.generation.next();
            let comp = self.comp.next();
            let objectId = self.objectId.next();
            let body = self.body.next();
            let player = self.player.next();
            let enemy = self.enemy.next();
            let animator = self.animator.next();
            if generation == None {
                done = true;
                return None;
            } else {
                let c = comp.unwrap();
                if c.1.test(self.mask) {
                    done = true;
                    return Some(
                        CpReference {
                            entity: Entity::from(c.0.try_into().unwrap(), *generation.unwrap().1),
                            comp: c.1,
                            objectId: objectId.unwrap().1,
                            body: body.unwrap().1,
                            player: player.unwrap().1,
                            enemy: enemy.unwrap().1,
                            animator: animator.unwrap().1,
                        }
                    );
                } else {
                    // do nothing, keep going!
                }
            }
        }
        None
    }
}


#[derive(Default, Copy, Clone)]
struct CpPrefab {
    comp: Cf,
    objectId: ObjType,
    body: Body,
    player: Player,
    enemy: Enemy,
    animator: Animator,
}

impl CpPrefab {

    fn get(&mut self, cp: &Cp, entity: Entity) {
        let i =  entity.index();
        if cp.pack.comp[i].test(Cf::Component) == true { self.comp = cp.pack.comp[i]; }
        if cp.pack.comp[i].test(Cf::ObjectId) == true {  self.objectId = cp.pack.objectId[i]; }
        if cp.pack.comp[i].test(Cf::Body) == true { self.body = cp.pack.body[i]; }
        if cp.pack.comp[i].test(Cf::Player) == true { self.player = cp.pack.player[i]; }
        if cp.pack.comp[i].test(Cf::Enemy) == true { self.enemy = cp.pack.enemy[i]; }
        if cp.pack.comp[i].test(Cf::Animator) == true { self.animator = cp.pack.animator[i]; }
    }

    fn set(&self, cp: &mut Cp, entity: Entity) {
        if cp.valid(entity) == true {
            let i =  entity.index();
            if self.comp.test(Cf::Component) == true { cp.pack.comp[i] = self.comp; }
            if self.comp.test(Cf::ObjectId) == true { cp.pack.objectId[i] = self.objectId; }
            if self.comp.test(Cf::Body) == true { cp.pack.body[i] = self.body; }
            if self.comp.test(Cf::Player) == true { cp.pack.player[i] = self.player; }
            if self.comp.test(Cf::Enemy) == true { cp.pack.enemy[i] = self.enemy; }
            if self.comp.test(Cf::Animator) == true { cp.pack.animator[i] = self.animator; }
        }
    }

}



#[derive(Default)]
struct Cp {
    manager: IndexTable,
    pack: CpPack,
}


impl Cp {

    fn new() -> Self {
        let mut cp:Cp = Default::default();
        cp.clear();
        cp
    }

    fn smartCopy(&mut self, other: &Self) {
        self.manager.list.resize(other.manager.list.len(), Default::default());   
        self.manager.head = other.manager.head;
        for i in 0..other.manager.list.len() { self.manager.list[i] = other.manager.list[i]; } 
        self.pack.smartCopy(&other.pack);
    }

    fn valid(&self, entity: Entity) -> bool {
        !entity.is_null() && entity.generation() == self.pack.generation[entity.index()]
    }

    fn destroy(&mut self, entity: Entity) {
        if self.valid(entity) == true {
            self.pack.generation[entity.index()] += 1;
            self.pack.comp[entity.index()].value = Cf::None;

            self.manager.free( entity.id() );
        }
    }

    fn create(&mut self) -> Entity {
        let mut entity = Entity::new();
        let value = self.manager.allocate();
        if value != IndexTable::END_OF_LIST {
            let deref:usize = value.into();
            self.pack.resize(deref + 1);
            self.pack.comp[deref].value = Cf::None;
            entity = Entity::from(value, self.pack.generation[deref])
        }
        entity
    }

    fn clear(&mut self) {
        self.manager.reset();
        self.pack.clear();
    }

    fn iter(&mut self) -> CpIterMut{
        self.pack.iter()
    }

    fn filter(&mut self, mask: u8) -> CpIterMut{
        self.pack.filter(mask)
    }

    fn print(&self, index: u16) -> bool {
        let mut ret = false;
        let i: usize = index.into();
        if i < self.pack.comp.len() {
            godot_print!("*********************************************");
            godot_print!("Entity Index: {}", index);
            godot_print!("Comp: {}", self.pack.comp[i].value);
            godot_print!("ObjectId: {}", self.pack.objectId[i].value);
            godot_print!("body position x: {}", self.pack.body[i].position.x);
            godot_print!("body position y: {}", self.pack.body[i].position.y);
            godot_print!("Enemy direction: {}", self.pack.enemy[i].direction);
            godot_print!("Enemy counter: {}", self.pack.enemy[i].counter);
            godot_print!("Enemy delayFire: {}", self.pack.enemy[i].delayFire);
            godot_print!("Animator frame: {}", self.pack.animator[i].frame);
            godot_print!("Animator counter: {}", self.pack.animator[i].count);

            if self.pack.comp[i].none() == true {
                ret = true;
            }
        }
        ret
    }



}


/***************************

    TheGame

**************************/

struct Data {
    prefabs: [CpPrefab; ObjType::USizeCount],
}


impl Data {
    const text_animate_counter: i32 = 8333333;
    const enemy_type_count: u16 = 11;

    const _null: u16 = 0;
    const _null_persist: u16 = 1;
    const enemy_00_a: u16 = 2;
    const enemy_01_a: u16 = 3;
    const enemy_02_a: u16 = 4;
    const enemy_03_a: u16 = 5;
    const enemy_04_a: u16 = 6;
    const enemy_05_a: u16 = 7;
    const enemy_06_a: u16 = 8;
    const enemy_07_a: u16 = 9;
    const enemy_08_a: u16 = 10;
    const enemy_09_a: u16 = 11;
    const enemy_10_a: u16 = 12;
    const enemy_00_b: u16 = 13;
    const enemy_01_b: u16 = 14;
    const enemy_02_b: u16 = 15;
    const enemy_03_b: u16 = 16;
    const enemy_04_b: u16 = 17;
    const enemy_05_b: u16 = 18;
    const enemy_06_b: u16 = 19;
    const enemy_07_b: u16 = 20;
    const enemy_08_b: u16 = 21;
    const enemy_09_b: u16 = 22;
    const enemy_10_b: u16 = 23;
    const player_ship_0: u16 = 24;
    const player_ship_1: u16 = 25;
    const player_shot: u16 = 26;
    const enemy_shot: u16 = 27;
    const easy_0: u16 = 28;
    const easy_1: u16 = 29;
    const player_boom_0: u16 = 30;
    const player_boom_1: u16 = 31;
    const player_boom_2: u16 = 32;
    const player_boom_3: u16 = 33;
    const player_boom_4: u16 = 34;
    const player_boom_5: u16 = 35;
    const player_boom_6: u16 = 36;
    const enemy_boom_0: u16 = 37;
    const enemy_boom_1: u16 = 38;
    const enemy_boom_2: u16 = 39;
    const enemy_boom_3: u16 = 40;
    const enemy_boom_4: u16 = 41;
    const enemy_boom_5: u16 = 42;
    const enemy_boom_6: u16 = 43;
    const local_player_0: u16 = 44;
    const local_player_1: u16 = 45;
    const text_ready: u16 = 46;
    const text_no: u16 = 47;
    const text_great: u16 = 48;
    const target: u16 = 49;
    const image_count: u16 = 50;
    const _end_list: u16 = 51;

    fn prefab(&self, index: u8) -> CpPrefab {
        self.prefabs[usize::try_from(index).unwrap()]
    }

    fn new() -> Self {
        Self {
            prefabs: [
                // null object
                CpPrefab {
                    comp: Cf::from(Cf::Component),
                    objectId: ObjType::from(ObjType::Null),
                    ..Default::default()
                },

                // player
                CpPrefab {
                    comp: Cf::from(Cf::Active | Cf::Component | Cf::Body | Cf::ObjectId | Cf::Player | Cf::Animator),
                    objectId: ObjType::from(ObjType::Player),
                    body: Body {
                        size: Vector2::from(16, 10),
                        ..Default::default()
                    },
                    player: Player {
                        slot: -1,
                        delayFire: 0,
                        damage: 0,
                    },
                    animator: Animator {
                        frame: Data::player_ship_0,
                        count: 0,
                    },
                    ..Default::default()
                },          

                // enemy
                CpPrefab {
                    comp: Cf::from(Cf::Active | Cf::Component | Cf::Body | Cf::ObjectId | Cf::Enemy | Cf::Animator),
                    objectId: ObjType::from(ObjType::Enemy),
                    body: Body {
                        size: Vector2::from(16, 10),
                        ..Default::default()
                    },
                    enemy: Enemy {
                        direction: 1,
                        counter: 0,
                        delayFire: 0,
                    },
                    animator: Animator {
                        frame: Data::player_ship_0,
                        count: 0,
                    },
                    ..Default::default()
                },

                // bullet
                CpPrefab {
                    comp: Cf::from(Cf::Active | Cf::Component | Cf::Body | Cf::ObjectId | Cf::Animator),
                    objectId: ObjType::from(ObjType::Bullet),
                    body: Body {
                        velocity: Vector2::from(0, 16),
                        size: Vector2::from(12, 20),
                        ..Default::default()
                    },
                    animator: Animator {
                        frame: Data::player_shot,
                        count: 0,
                    },
                    ..Default::default()
                },
                        
                // bad bullet
                CpPrefab {
                    comp: Cf::from(Cf::Active | Cf::Component | Cf::Body | Cf::ObjectId | Cf::Animator),
                    objectId: ObjType::from(ObjType::BadBullet),
                    body: Body {
                        velocity: Vector2::from(0, -8),
                        size: Vector2::from(7, 7),
                        ..Default::default()
                    },
                    animator: Animator {
                        frame: Data::enemy_shot,
                        count: 0,
                    },
                    ..Default::default()
                },

                 // boom
                 CpPrefab {
                    comp: Cf::from(Cf::Active | Cf::Component | Cf::Body | Cf::ObjectId | Cf::Animator),
                    objectId: ObjType::from(ObjType::Boom),
                    body: Body {
                        size: Vector2::from(14, 14),
                        ..Default::default()
                    },
                    animator: Animator {
                        frame: Data::enemy_boom_0,
                        count: 0,
                    },
                    ..Default::default()
                },

                // player boom
                CpPrefab {
                    comp: Cf::from(Cf::Active | Cf::Component | Cf::Body | Cf::ObjectId | Cf::Animator),
                    objectId: ObjType::from(ObjType::PlayerBoom),
                    body: Body {
                        size: Vector2::from(22, 21),
                        ..Default::default()
                    },
                    animator: Animator {
                        frame: Data::player_boom_0,
                        count: 0,
                    },
                    ..Default::default()
                },

                // shot cleaner
                CpPrefab {
                    comp: Cf::from(Cf::Active | Cf::Component | Cf::Body | Cf::ObjectId),
                    objectId: ObjType::from(ObjType::ShotCleaner),
                    body: Body {
                        size: Vector2::from(960, 540),
                        ..Default::default()
                    },
                    ..Default::default()
                },

            ],
        }
    }

}



struct Game {
    //collision_table: HashMap<u16, GameCallback>,
    animation_table: HashMap<u16, u16>,

    data: Data, // make this a reference?

    slots: Vec<Slot>,

    global: GlobalState,
    rand: MersenneTwister,
    components: Cp,

    boundList: Vec<Bounds>,
    eventList: Vec<Event>,

    gameOver: bool,
}

impl Game {

    const SyncedMask:BitMask = BitMask::const_from(55, 56);
    const XMask:BitMask = BitMask::const_from(0, 16);
    const LeftMask:BitMask = BitMask::const_from(16, 17);
    const RightMask:BitMask = BitMask::const_from(17, 18);
    const PrimaryMask:BitMask = BitMask::const_from(18, 19);

    fn new() -> Self {
        Self {
            //collision_table: Game::setupCollisionTable(),
            animation_table: Game::setupAnimationTable(),

            data: Data::new(),
            slots: vec![Default::default(); 64],
            
            global: Default::default(),

            rand: MersenneTwister::new(),
            components: Cp::new(),

            boundList: Vec::new(),
            eventList: Vec::new(),

            gameOver: false,
        }
    }

    // it's interesting that I can't use this function
    //fn iter_filter(&mut self, mask: u8) -> CpIterMut {
    //    self.components.iter_filter(mask)
    //}

    const fn computeKey(a: u8, b: u8) -> u16 { ((a as u16)  << 8) | (b as u16) }   
    const shotCleanerVsBulletKey: u16 = Game::computeKey(ObjType::ShotCleaner, ObjType::Bullet);
    const shotCleanerVsBadBulletKey: u16 = Game::computeKey(ObjType::ShotCleaner, ObjType::BadBullet);
    const bulletVsEnemyKey: u16 = Game::computeKey(ObjType::Bullet, ObjType::Enemy);
    const badBulletVsPlayerKey: u16 = Game::computeKey(ObjType::BadBullet, ObjType::Player);
    fn collisionFunction(game: &mut Self, index: usize) {
        let e = game.eventList[index];
        let mut iter = game.components.iter();
        let (a, b) = iter.nth_double(e.a, e.b).unwrap();
        match e.key {
            Game::shotCleanerVsBulletKey | 
            Game::shotCleanerVsBadBulletKey => {
                // do colision betweem e.A bullet, and e.B enemy!
                if b.animator.frame != Data::_null
                {
                    b.animator.frame = Data::_null;
                }
            },
            Game::bulletVsEnemyKey => {
                // do colision betweem e.A bullet, and e.B enemy!
                if a.animator.frame != Data::_null && b.animator.frame != Data::_null
                {
                    a.animator.frame = Data::_null;
                    b.animator.frame = Data::_null;
                    game.eventList.push( Event::from_entity(ObjType::Boom, b.body.position ) );
                }
            },
            Game::badBulletVsPlayerKey => {
                 // do colision betweem e.A bullet, and e.B enemy!
                 if a.animator.frame != Data::_null && b.animator.frame != Data::_null_persist
                {
                    a.animator.frame = Data::_null;

                    b.player.damage = 100;
                    b.animator.frame = Data::_null_persist;

                    game.eventList.push( Event::from_entity(ObjType::PlayerBoom, b.body.position ) );
                }
            },
            _ => {
                // no logic for collision!
            }
        }
    }
    
    
    fn setupAnimationTable() -> HashMap<u16, u16> {
        let mut hash = HashMap::new();

        // Animations
        let player_boom = [
            Data::player_boom_0,
            Data::player_boom_1,
            Data::player_boom_2,
            Data::player_boom_3,
            Data::player_boom_4,
            Data::player_boom_5,
            Data::player_boom_6,
            Data::_null,
            Data::_end_list
        ];
        Game::registerAnimation(&mut hash, &player_boom);

        let enemy_boom = [
            Data::enemy_boom_0,
            Data::enemy_boom_1,
            Data::enemy_boom_2,
            Data::enemy_boom_3,
            Data::enemy_boom_4,
            Data::enemy_boom_5,
            Data::enemy_boom_6,
            Data::_null,
            Data::_end_list
        ];
        Game::registerAnimation(&mut hash, &enemy_boom);

        let player = [
            Data::player_ship_0,
            Data::player_ship_1,
            Data::_end_list
        ];
        Game::registerAnimation(&mut hash, &player);

        let local_player = [
            Data::local_player_0,
            Data::local_player_1,
            Data::_end_list
        ];
        Game::registerAnimation(&mut hash, &local_player);

        let mut enemy = [
            Data::_null,
            Data::_null,
            Data::_end_list
        ];
        for i in 0..Data::enemy_type_count {
            let i_u16 = u16::try_from(i).unwrap();
            enemy[0] = i_u16 + 2;
            enemy[1] = i_u16 + 2 + Data::enemy_type_count;
            Game::registerAnimation(&mut hash, &enemy);
        }

        hash
    }

    fn registerAnimation(hash: &mut HashMap<u16, u16>, list: &[u16]) {
        let mut i = 0;
        while list[i] != Data::_end_list {
            if list[i] != Data::_null {
                hash.insert(list[i], list[i + 1]);
            }
            i += 1;
        }
        let last = list[i - 1];
        if last != Data::_null {
            hash.insert(last, list[0]);
        }
    }
    
    
    fn smartCopy(&mut self, other: &Game) {
        self.slots.resize(other.slots.len(), Default::default());
        for i in 0..other.slots.len() { self.slots[i] = other.slots[i]; }

        self.global = other.global;
        self.rand = other.rand;
        self.components.smartCopy(&other.components);
        self.gameOver = other.gameOver;
    }
    

    
    fn init(&mut self, seed: u32) {

        self.rand.set_seed(seed);

        self.gameOver = false; // do I need this now?

        self.global.playing = false;
        self.global.enemySpeed = 3; // 3
        self.global.enemyCount = 0;
        self.global.textType = Data::text_ready;
        self.global.textAnimate = 0;

        self.components.clear();

        self.eventList.clear();
        for usize_j in 0..self.slots.len() {
            // if slot is connected
            if self.slots[usize_j].connected == true && self.slots[usize_j].connectedAtStart == true
            {
                let j = i32::try_from(usize_j).unwrap();
                let j16 = u16::try_from(usize_j).unwrap();
                self.eventList.push( Event::from_player(j16, Vector2::from(j * 60 - 960 + 32, -500) ) );
            }
        }

        // shot cleaners make sure shots don't last forever
        self.eventList.push( Event::from_entity(ObjType::ShotCleaner, Vector2::from(0,  1090) ) );
        self.eventList.push( Event::from_entity(ObjType::ShotCleaner, Vector2::from(0, -1090) ) );

        self.resolveEvents();

    }

    fn compute_input(&mut self) {
        for slot in &mut self.slots {
            slot.input.primary = Game::PrimaryMask.decode(slot.raw) == 1;
            slot.input.left = Game::LeftMask.decode(slot.raw) == 1;
            slot.input.right = Game::RightMask.decode(slot.raw) == 1;
            // ignore for now
            //slot.Input.X = XMask.decode(slot.raw);
        }
    }

    fn resolveState(&mut self) {
        if self.global.playing == false
        {
            self.global.textAnimate += Data::text_animate_counter;

            if self.global.textAnimate > 1000000000
            {
                // this was for when the game actually quit back to the lobby!
                if self.global.textType != Data::text_ready
                {
                    self.gameOver = true;
                }
                else
                {
                    self.global.playing = true;

                    // fix all ships
                    //Targets.Clear();

                    for r in self.components.pack.filter(Cf::Active | Cf::Player) {
                        if self.slots[usize::try_from(r.player.slot).unwrap()].connected == true {
                            r.player.damage = 0;
                            r.animator.frame = Data::player_ship_0;
                        }
                    }

                    // j 24, i 20
                    // create new set of entities
                    for j_usize in 0..24
                    {
                        for i_usize in 0..20
                        {
                            let i = i32::try_from(i_usize).unwrap();
                            let j = i32::try_from(j_usize).unwrap();
                            self.eventList.push( Event::from_entity(ObjType::Enemy, Vector2::from(j * 60 - 960 + 32, i * 32 - 100) ) );
                        }
                    }
                }
            }
        }
    }
    

    fn update(&mut self) {
        self.compute_input();
        self.updateAnimators();
        self.updatePlayers();
        self.updateEnemies();        
        self.integrate();
        self.resolveState();   
        self.fillContactList();
        self.resolveEvents();
    }

    fn fastForward(&mut self) {
        self.compute_input();
        self.updateAnimators();
        self.updatePlayers();
        self.updateEnemies();
        self.integrate();
        self.resolveState();
    }

    
    fn fillContactList(&mut self) {
        // clear the bounds list
        self.boundList.clear();

        // fill up the bounds list with objects
        for r in self.components.filter(Cf::Active | Cf::Body) {
            self.boundList.push( Bounds::from(r.entity, r.objectId.value, r.body.position, r.body.velocity, r.body.size) );
        }

        // sort it here!
        //self.boundList.sort();

        self.boundList.sort_by(|a, b| {
            if a.lower.x < b.lower.x {
                Ordering::Less
            } else if a.lower.x == b.lower.x {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        });

        // collision overlap code
        let len = self.boundList.len();
        for i in 0..len {
            let iter = &self.boundList[i];
            for j in (i + 1)..len {
                let nextIter = &self.boundList[j];
                if iter.overlap(&nextIter) == true {
                    let it = u16::try_from(iter.btype).unwrap();
                    let nxit = u16::try_from(nextIter.btype).unwrap();
                    if iter.btype > nextIter.btype {
                        self.eventList.push( Event::from_contact( (it << 8) | nxit, iter.entity, nextIter.entity ) );
                    } else {
                        self.eventList.push( Event::from_contact( (nxit << 8) | it, nextIter.entity, iter.entity ) );
                    }                    
                }
            }
        }

    }
    

    fn integrate(&mut self) {
        for r in self.components.filter(Cf::Active | Cf::Body) {
            r.body.position += r.body.velocity;
        }
    }

    
    fn updateAnimators(&mut self) {
        for r in self.components.filter(Cf::Active | Cf::Animator) {
            r.animator.count += 1;
            if r.animator.count > 3 {
                r.animator.count = 0;
                let find_it = self.animation_table.get(&r.animator.frame);
                if find_it != None {
                    r.animator.frame = *(find_it.unwrap());
                }
            }
            if r.animator.frame == Data::_null {
                self.eventList.push( Event::from_destroy(r.entity) );
            }
        }
    }


    fn updateEnemies(&mut self) {
        self.global.enemyCount = 0;

        for r in self.components.filter(Cf::Active | Cf::Body | Cf::Enemy) {
            r.enemy.counter += 1;
            if r.enemy.counter > 150 {
                r.enemy.counter = 0;
                r.enemy.direction = -r.enemy.direction;
            }

            r.body.velocity.x = i32::try_from(r.enemy.direction).unwrap() * self.global.enemySpeed;

            if r.enemy.delayFire > 0 {
                r.enemy.delayFire -= 1;
            }

            if r.enemy.delayFire == 0 {
                r.enemy.delayFire = 2000;
                self.eventList.push( Event::from_entity(ObjType::BadBullet, r.body.position ) );
            }

            self.global.enemyCount += 1;
        }
        // calculate enemy speed based on count
        self.global.enemySpeed = 3;

        // if enemy count is zero, set playing to false, text to great job!
        if self.global.playing == true && self.global.enemyCount == 0 {
            self.global.playing = false;
            self.global.textAnimate = 0;
            self.global.textType = Data::text_great;
            // also repair all ships!
            // actually this is done by the reset function!
        }
    }

    fn updatePlayers(&mut self) {
        //Targets.Clear();
        let mut livePlayer = false;

        for r in self.components.filter(Cf::Active | Cf::Body | Cf::Player) {

            let slot = self.slots[usize::try_from(r.player.slot).unwrap()];

            r.body.velocity.x = 0;
            if r.player.delayFire > 0 {
                r.player.delayFire -= 1;
            }

            if r.player.damage > 0
            {
                // DO NOTHING!
            }
            else
            {
                if slot.connected == true
                {
                    livePlayer = true;
                }
                else
                {
                    // kill if disconnect!
                    r.player.damage = 100;
                    r.animator.frame = Data::_null_persist;
                    self.eventList.push( Event::from_entity(ObjType::PlayerBoom, r.body.position ) );
                }

                if r.body.position.x < -960 { r.body.position.x = -960; }
                if r.body.position.x > 960 { r.body.position.x = 960; }

                if slot.input.left == true { r.body.velocity.x = -5; }
                if slot.input.right == true { r.body.velocity.x = 5; }

                if slot.input.primary && r.player.delayFire == 0
                {
                    r.player.delayFire = 24;
                    self.eventList.push( Event::from_entity(ObjType::Bullet, r.body.position ) );
                }

            }

        }

        // if all players are damaged, set playing to false, text to no way! and destroy all enemies
        
        if self.global.playing == true && livePlayer == false
        {
            self.global.playing = false;
            self.global.textAnimate = 0;
            self.global.textType = Data::text_no;

            for r in self.components.filter(Cf::Active | Cf::Body | Cf::Enemy) {
                self.eventList.push( Event::from_destroy(r.entity) );
            }
        }
        
    }
    

    fn resolveEvents(&mut self) {
        let mut event_index = 0;
        while event_index < self.eventList.len() {      
            match self.eventList[event_index].id {
                Event::Contact => {
                    // not using a hash table, and need to get better understanding
                    // on how nested references work
                    if self.components.valid(self.eventList[event_index].a) == true && 
                       self.components.valid(self.eventList[event_index].b) == true {
                        Game::collisionFunction(self, event_index);
                    }
                },
                Event::DestroyEntity => {
                    self.components.destroy(self.eventList[event_index].a);
                },
                Event::CreateEntity => {
                    // create from prefab!
                    let entity = self.components.create();
                    self.data.prefab(self.eventList[event_index].otype).set(&mut self.components, entity);
                    let mut iter = self.components.pack.iter();
                    let r = iter.nth(entity).unwrap();

                    if !entity.is_null() {
                        if r.comp.test(Cf::Body) == true {
                            r.body.position = self.eventList[event_index].v;
                        }
                        if r.comp.test(Cf::Player) == true {
                            r.player.slot = i8::try_from(self.eventList[event_index].key).unwrap();
                        }
                        if r.comp.test(Cf::Enemy) == true {
                            let count = i32::try_from(Data::enemy_type_count).unwrap();
                            //it.enemy.delayFire = u16::try_from(self.rand.next_from_zero(2000)).unwrap();
                            r.enemy.delayFire = u16::try_from(self.rand.next_u32() % 2000).unwrap();
                            r.animator.frame = u16::try_from(self.rand.next_from_zero(count) + 2).unwrap();
                        }
                    }           

                }
                _ => {
                    // do nothing, better yet, panic!
                }
            }
            event_index += 1;
        }

        self.eventList.clear();
    
    }
    


}



/*******************************************************************

    GDNativeScript (Everything above should go into a 'Game' crate!)

********************************************************************/


/// The HelloWorld "class"
#[derive(NativeClass)]
#[inherit(Node)]
pub struct HelloWorld {
    local_player: i8,
    game: Game,
}

// You may add any number of ordinary `impl` blocks as you want. However, ...
impl HelloWorld {
    /// The "constructor" of the class.
    fn new(_base: &Node) -> Self {
        Self {
            local_player: 0,  
            game: Game::new(),
        }
    }
}


// Only __one__ `impl` block can have the `#[methods]` attribute, which
// will generate code to automatically bind any exported methods to Godot.
#[methods]
impl HelloWorld {

    // To make a method known to Godot, use the #[method] attribute.
    // In Godot, script "classes" do not actually inherit the parent class.
    // Instead, they are "attached" to the parent object, called the "base".
    //
    // If access to the base instance is desired, the 2nd parameter can be
    // annotated with #[base]. It must have type `&T` or `TRef<T>`, where `T`
    // is the base type specified in #[inherit]. If you don't need this parameter,
    // feel free to omit it entirely.
    #[method]
    fn _ready(&mut self, #[base] base: &Node) {
        // The `godot_print!` macro works like `println!` but prints to the Godot-editor
        // output tab as well.
        godot_print!("Hello world from node {}!", base.to_string());
    }

    #[method]
    fn custom_game_over(&self) -> bool {
        self.game.gameOver
    }

    #[method]
    fn custom_init(&mut self, l:i64, r:i64) {
        self.local_player = i8::try_from(l).unwrap();
        self.game.init( u32::try_from(r).unwrap() );

        godot_print!("I initailzed!");
    }

    #[method]
    fn custom_set_input(&mut self, index:usize, connected:bool, start:bool, broken:bool, raw:i64) {
        self.game.slots[index].connected = connected;
        self.game.slots[index].connectedAtStart = start;
        self.game.slots[index].broken = broken;
        self.game.slots[index].raw = raw;
    }

    #[method]
    fn custom_update(&mut self) {
        // update game here
        self.game.update();
    }

    #[method]
    fn custom_render(&mut self, layer_ref: Ref<Node>) {
        
        

        // The `godot_print!` macro works like `println!` but prints to the Godot-editor
        // output tab as well.

        // start rendering here
        let layer:TRef<Node> = unsafe { layer_ref.assume_safe() };
      
        unsafe { layer.call("clear_sprites", &[]) };


        let mut draw: bool = false;
        let mut tx: i32 = 0;
        let mut ty: i32 = 0;
        let mut tf: u16 = 0;

        for r in self.game.components.filter(Cf::Active | Cf::Component | Cf::Body | Cf::ObjectId | Cf::Animator) {
       
            draw = true;

            if r.comp.test(Cf::Player) == true
            {
                if r.player.slot == self.local_player
                {
                    draw = false;
                    tx = r.body.position.x;
                    ty = r.body.position.y;
                    tf = r.animator.frame;
                }
            }
            if draw == true
            {
                //let f = self.game.rand.next_range(0, 49).to_variant();
                //let x = self.game.rand.next_range(-480, 480).to_variant();
                //let y = self.game.rand.next_range(-271, 271).to_variant();

                let f = r.animator.frame.to_variant();
                let x = r.body.position.x.to_variant();
                let y = (-r.body.position.y).to_variant();
                unsafe { layer.call("create_invader", &[f, x, y, 2.to_variant(), 2.to_variant()]) };
            }

        }

        // quick hack to get local player rendering different
        {
            if tf == Data::player_ship_0 { tf = Data::local_player_0; }
            if tf == Data::player_ship_1 { tf = Data::local_player_1; }
            let f = tf.to_variant();
            let x = tx.to_variant();
            let y = (-ty).to_variant();
            let sx = 2.to_variant();
            let sy = 2.to_variant();
            unsafe { layer.call("create_invader", &[f, x, y, sx, sy]) };
        }

        // render text!
        if self.game.global.playing == false
        {
            //float textScale = 900.0f * (float)game.State.textAnimate;
            //        0_008333333
            // Const.v0_00833333333333333

            let tx: f32 = self.game.global.textAnimate as f32 / 1000000000.0;
            let c1: f32 = 1.70158;
            let c3: f32 = c1 + 1.0;

            //godot_print!("the amount {}!", tx);

            //float textScale = 1 + c3 * (float)Math.Pow(x - 1, 3) + c1 * (float)Math.Pow(x - 1, 2);
            //let mut textScale = 1.0 - f32::powf(1.0 - tx, 3.0);
            //let mut textScale = 1.0 - 3i32.pow(1.0 - tx);
            let mut textScale = 1.0 - f32::powf(1.0 - tx, 3.0);
            
            textScale *= 8.0;

            //textScale = 2.0;

            let f = self.game.global.textType.to_variant();
            let x = 0.to_variant();
            let y = 0.to_variant();
            let sx = textScale.to_variant();
            let sy = textScale.to_variant();
 
            unsafe { layer.call("create_invader", &[f, x, y, sx, sy]) };
        }
        
        

    }

}

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    // Register the new `HelloWorld` type we just declared.
    handle.add_class::<HelloWorld>();
}

// Macro that creates the entry-points of the dynamic library.
godot_init!(init);