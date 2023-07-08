mod double;
use double::DLList;

fn main() {
    let mut a = DLList::new();
    println!("{a:?}");
    a.append(8);
    println!("{a:?}");
    a.prepend(16);
    println!("{a:?}");
    a.insert_at(32, 1);
    println!("{a:?}");
    a.insert_at(64, 1);
    println!("{a:?}");
    a.remove_at(2);
    println!("{a:?}");
    a.remove_at(2);
    println!("{a:?}");
}
