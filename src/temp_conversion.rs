type TempPair = (u16, u16);

static LOOKUP_TABLE : &[TempPair]= &[
    (0, 10000),
    (1000, 10390),
    (2000, 10779),
    (3000, 11167),
    (4000, 11554),
    (5000, 11940),
    (6000, 12324),
    (7000, 12708),
    (8000, 13090),
    (9000, 13471),
    (10000, 13851),
    (11000, 14229),
    (12000, 14607),
    (13000, 14983),
];

pub fn lookup_temperature(val : u16) -> u32 {
    let mut first = &(0, 10000);
    let mut second = &(1000, 10390);
    let mut iterator = LOOKUP_TABLE.iter();
    while let Some(a) = iterator.next() {
        first = second;
        second = &a;
        if a.1 > val { break; }
    }
    let second = iterator.next();

    if let Some(second) = second {
        let temp = (second.0 - first.0) as u32 * (val - first.1) as u32 / (second.1 - first.1) as u32 + first.0 as u32;
        temp
    } else {
        0
    }
}