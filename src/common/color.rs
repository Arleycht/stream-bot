use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

pub const DEFAULT_COLORS: [u32; 15] = [
    0xFF0000, 0x0000FF, 0x008000, //
    0xB22222, 0xFF7F50, 0x9ACD32, //
    0xFF4500, 0x2E8B57, 0xDAA520, //
    0xD2691E, 0x5F9EA0, 0x1E90FF, //
    0xFF69B4, 0x8A2BE2, 0x00FF7F, //
];

type UserColorCache = HashMap<String, u32>;

static HASHMAP: RwLock<OnceLock<UserColorCache>> = RwLock::new(OnceLock::new());

fn get_random_user_color() -> u32 {
    DEFAULT_COLORS[fastrand::usize(..DEFAULT_COLORS.len())]
}

pub fn get_user_color(username: &String) -> u32 {
    let guard = HASHMAP.read().unwrap();
    let map = guard.get_or_init(HashMap::new);

    if map.contains_key(username) {
        *map.get(username).unwrap()
    } else {
        // Drop initial read guard because we need to mutate the value
        drop(guard);
        set_user_color(username, get_random_user_color())
    }
}

pub fn set_user_color(k: &String, color: u32) -> u32 {
    let mut guard = HASHMAP.write().unwrap();
    let map = guard.get_mut().unwrap();
    map.insert(k.clone(), color);
    color
}

#[cfg(test)]
mod unit_tests {
    use super::get_user_color;

    #[test]
    fn user_colors() {
        // Set test seed
        fastrand::seed(12);

        let user1 = "UserA".to_string();
        let user2 = "UserB".to_string();

        let color1 = get_user_color(&user1);
        let color2 = get_user_color(&user1);
        let color3 = get_user_color(&user2);

        assert!(color1 == color2);
        assert!(color1 != color3);
    }
}
