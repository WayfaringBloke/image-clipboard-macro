use inputbot;
use global::Global;

static BINDS: Global<data::ImageBinds> = Global::new();

pub fn run() {
    BINDS.force_init();
    binds::bind();
    inputbot::handle_input_events();
}

mod clipboard {
    use clipboard_win::{formats, set_clipboard, get_clipboard};
    use inputbot::KeybdKey;
    use crate::data;

    impl data::ImageBinds {
        pub fn add_img(&mut self, key: KeybdKey) {
            match get_clipboard(formats::Bitmap) {
                Ok(bmp) =>  {
                    self.binds_insert(key, bmp);
                    if let Err(e) = self.save() {
                        println!("couldn't save binds: {}", e)
                    } else {
                        println!("image saved to {:?}", key)
                    };
                }
                Err(e) => println!("failed to get clipboard: {}", e)
            };
        }
        pub fn paste_img(&self, key: KeybdKey) {
            if let Err(e) = set_clipboard(formats::Bitmap, self.get_img(&key)) {
                println!("couldn't set clipboard for {:?}: {}", key, e)
            } else {
                println!("clipboard set to {:?} image", key)
            };
        }
    }
}

mod file {
    use crate::data;
    use bincode;
    use std::{fs::OpenOptions, io::Write, error::Error};
    pub static BIN_PATH: &str = "data.bin";

    impl data::ImageBinds {
        pub fn save(&self) -> Result<(), Box<dyn Error>>{
            let mut f = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(BIN_PATH)?;
            let bin = bincode::serialize(&self)?;
            f.write_all(&bin)?;
            Ok(())
        }

        pub fn load() ->  Result<data::ImageBinds, Box<dyn Error>> {
            let f = OpenOptions::new()
                .read(true)
                .open("data.bin")?;
            let binds = bincode::deserialize_from(f)?;
            Ok(binds)
        }
    }
}


mod data {
    use inputbot::KeybdKey;
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize};
    use crate::file;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ImageBinds {
        binds: HashMap<u64, Vec<u8>>
    }


    impl Default for ImageBinds {
        fn default() -> Self {
            match ImageBinds::load() {
                Ok(binds) => {
                    println!("loaded binds from {}", file::BIN_PATH);
                    binds
                }
                Err(e) => {
                    println!("couldn't load binds from {}: {}", file::BIN_PATH, e);
                    Self { binds: HashMap::new() }
                }
            }
        }
    }

    impl ImageBinds {
        pub fn get_img(&self, &key: &KeybdKey) -> &Vec<u8>{
            self.binds.get(&u64::from(key)).expect("Trying to paste an image that doesn't exist")
        }
        pub fn binds_insert(&mut self, k: KeybdKey, v: Vec<u8>) {
            self.binds.insert(u64::from(k), v);
        }

        pub fn get_keys(&self) -> Vec<KeybdKey> {
            self.binds
                .keys()
                .map(|n: &u64| -> KeybdKey {
                    KeybdKey::from(*n)
                })
                .collect()
        }
    }
}

mod binds {
    use inputbot::{KeybdKey::{LControlKey, JKey, LShiftKey}};
    use std::{time::Duration, thread::sleep};
    use global::Global;

    static LCTRL_LISTENING: Global<bool> = Global::new();

    pub fn bind() {
        LControlKey.bind(|| {
            match LCTRL_LISTENING.lock_mut() {
                Ok(mut b) => if *b == true { return } else { *b = true },
                Err(_) => return
            }
            if JKey.is_pressed() {
                return
            }

            while LControlKey.is_pressed() {
                if JKey.is_pressed() {
                    if LShiftKey.is_pressed() {
                        custom_keys::add_new_img();
                    } else {
                        custom_keys::wait_for_img();
                    }
                    break
                }
                sleep(Duration::from_millis(100));
            };

            if let Ok(mut b) = LCTRL_LISTENING.lock_mut() {
                *b = false;
            }
        })
    }
    mod alphabet {
        use inputbot::KeybdKey::{self, *};
        pub static KEYS: &[KeybdKey] = &[ 
            AKey, BKey, CKey, DKey, EKey, FKey,
            GKey, HKey, IKey, KKey, LKey, MKey, 
            NKey, OKey, PKey, QKey, RKey, SKey, 
            TKey, UKey, VKey, WKey, XKey, YKey, 
            ZKey];
    }
    mod custom_keys {
        use std::time::Instant;
        use crate::BINDS;
        use super::*;

        pub fn add_new_img() {
            let start = Instant::now();
            let wait_for = Duration::from_secs(4);
            while start.elapsed() < wait_for {
                let pressed = super::alphabet::KEYS.iter()
                    .filter(|elem| { elem.is_pressed() });
                if let Some(key) = pressed.last()  {
                    if let Ok(mut binds) = BINDS.lock_mut() {
                        binds.add_img(*key)
                    }
                    break
                    
                }
                sleep(Duration::from_millis(100))
            }
        }
        pub fn wait_for_img() {
            let start = Instant::now();
            let wait_for = Duration::from_secs(4);
            while start.elapsed() < wait_for {
                let binds = match BINDS.lock() {
                    Ok(d) => d,
                    Err(_) => break
                };
                let keys = binds.get_keys();
                let pressed = keys.iter()
                    .filter(|elem| { elem.is_pressed() });  

                if let Some(key) = pressed.last()  {
                    binds.paste_img(*key);
                    break
                };
                sleep(Duration::from_millis(100))
            }
        }
    }
}