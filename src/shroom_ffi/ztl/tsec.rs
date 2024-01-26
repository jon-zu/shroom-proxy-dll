// TODO: TSecData with size_of<T> smaller than 4 uses packing

#[repr(C, packed)]
pub struct TSecDataPacked<T> {
    data: T,
    key: u8,
    fake_ptr1: u8,
    fake_ptr2: u8,
    checksum: u16,
}

#[repr(C)]
pub struct TSecData<T> {
    data: T,
    keys: [u8; 3],
    checksum: u16,
}

// TODO: i8, u8
static_assertions::assert_eq_size!(TSecData<i32>, [u8; 0xC]);
static_assertions::assert_eq_size!(TSecData<f64>, [u8; 0x10]);

pub trait TSecTypeConverter: Default {
    type Bytes: AsRef<[u8]> + AsMut<[u8]>;
    fn to_bytes(&self) -> Self::Bytes;
    fn from_bytes(bytes: Self::Bytes) -> Self;
}

impl TSecTypeConverter for i32 {
    type Bytes = [u8; 4];
    fn to_bytes(&self) -> Self::Bytes {
        self.to_le_bytes()
    }
    fn from_bytes(bytes: Self::Bytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl<T: TSecTypeConverter> TSecData<T> {
    pub fn encrypt(&mut self, data: T) {
        let mut key: u8 = 11; //TODO: add rand
        self.keys[0] = key;
        let mut checksum: u16 = 39525;

        let mut data = data.to_bytes();

        for b in data.as_mut().iter_mut() {
            if key == 0 {
                key = 42;
            }
            *b ^= key;
            key = key.wrapping_add(*b).wrapping_add(42);
            checksum = (checksum >> 0xd).wrapping_add(key as u16) | (checksum << 3);
        }

        self.checksum = checksum;
        self.data = T::from_bytes(data);
    }

    pub fn decrypt(&self) -> Option<T> {
        let mut key = self.keys[0];
        let mut checksum = self.checksum;
        let mut data = self.data.to_bytes();

        for b in data.as_mut().iter_mut() {
            if key == 0 {
                key = 42;
            }
            *b ^= key;
            key = key.wrapping_add(*b).wrapping_add(42);
            checksum = (checksum >> 0xd).wrapping_add(key as u16) | (checksum << 3);
        }

        if checksum == self.checksum {
            Some(T::from_bytes(data))
        } else {
            None
        }
    }

    pub fn set_fake_ptrs(&mut self, ptr1: u32, ptr2: u32) {
        self.keys[1] = ptr1 as u8;
        self.keys[2] = ptr2 as u8;
    }
}

#[repr(C)]
pub struct TSecType<T> {
    fake_ptr1: u32,
    fake_ptr2: u32,
    sec_data: *mut TSecData<T>,
}

impl<T: TSecTypeConverter> TSecType<T> {
    pub fn create(data: T) -> Self {
        let sec_data = TSecData {
            data: T::default(),
            keys: [0; 3],
            checksum: 0,
        };

        let mut sec = Self {
            fake_ptr1: 0,
            fake_ptr2: 0,
            sec_data: Box::into_raw(Box::new(sec_data)),
        };
        sec.set(data);
        sec
    }

    pub fn get(&self) -> Option<T> {
        //TODO verify keys maybe
        unsafe { self.sec_data.as_ref() }.and_then(|d| d.decrypt())
    }

    pub fn set(&mut self, data: T) {
        let sec_data = unsafe { self.sec_data.as_mut() }.unwrap();
        sec_data.encrypt(data);
        sec_data.set_fake_ptrs(self.fake_ptr1, self.fake_ptr2);
    }
}
