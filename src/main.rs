#![forbid(unsafe_code)]
#![forbid(unstable_features)]
#![forbid(missing_fragment_specifier)]
#![warn(clippy::all, clippy::pedantic)]
use rand::rngs::ThreadRng;
use rand::Rng;
use std::io::Write;
use std::str::FromStr;

type Word = String;
type Consonant = char;
type Vowel = char;
type Syllable = String;

#[derive(Debug)]
struct Consonants(Vec<Consonant>);
#[derive(Debug)]
struct Vowels(Vec<Vowel>);

#[derive(Debug)]
struct SyllableGenerator {
    order: SyllableOrder,
    consonants: Vec<Consonant>,
    vowels: Vec<Vowel>,
    nasal: Vec<char>,
    syllable: Vec<LetterGenerator>,
    rng: ThreadRng,
    seeded: bool,
    index: usize,
}

impl SyllableGenerator {
    pub fn new(
        order: SyllableOrder,
        consonants: Vec<Consonant>,
        vowels: Vec<Vowel>,
        nasal: Vec<char>,
    ) -> SyllableGenerator {
        let rng = rand::thread_rng();

        Self {
            order,
            consonants,
            vowels,
            nasal,
            syllable: Vec::new(),
            rng,
            seeded: false,
            index: 0,
        }
    }

    pub fn iterate(&mut self) {
        if self.syllable.is_empty() {
            self.create_syllable();
        }

        self.seed();

        if self.is_collapsed() {
            // eprintln!("syllable iteration:\n{}", self.extract());
            return;
        }

        if self.index == self.syllable.len() {
            self.index = 0;
        }

        self.syllable[self.index].select(
            &mut self.rng,
            &self.consonants,
            &self.vowels,
            &self.nasal,
        );

        self.index += 1;
        // eprintln!("syllable iteration:\n{}", self.extract());
    }

    fn list(&self) -> Vec<char> {
        let mut list = Vec::new();

        list.append(&mut self.consonants.clone());
        list.append(&mut self.vowels.clone());
        list.append(&mut self.nasal.clone());

        list
    }

    pub fn extract(&mut self) -> Syllable {
        let mut chars: Vec<char> = Vec::new();

        for letter in &self.syllable {
            let mut letters = letter.get_letters();
            chars.append(&mut letters);
        }

        chars.iter().collect()
    }

    fn seed(&mut self) {
        if self.seeded {
            return;
        }

        if let Some(start) = self.get_random_uncollapsed() {
            self.syllable[start].select(&mut self.rng, &self.consonants, &self.vowels, &self.nasal);
            self.seeded = true;
        }
    }

    fn get_random_uncollapsed(&mut self) -> Option<usize> {
        if self.is_collapsed() {
            return None;
        }

        let len = self.syllable.len();
        loop {
            let index = self.rng.gen_range(0..len);

            if self.syllable[index].is_collapsed() {
                continue;
            }

            return Some(index);
        }
    }

    fn create_syllable(&mut self) {
        let mut syl: Vec<LetterGenerator> = Vec::new();

        for s in &self.order.0 {
            if self.rng.gen_bool(s.probability()) {
                syl.insert(syl.len(), LetterGenerator::new(*s, self.list()));
            }
        }

        self.syllable = syl;
    }

    pub fn is_collapsed(&self) -> bool {
        if self.seeded {
            for letter in &self.syllable {
                if !letter.is_collapsed() {
                    return false;
                }
            }
            return true;
        }
        false
    }
}

#[derive(Debug)]
struct LetterGenerator {
    kind: SyllableLetter,
    possible: Vec<char>,
    pruned: bool,
}

impl LetterGenerator {
    pub fn new(kind: SyllableLetter, list: Vec<char>) -> Self {
        Self {
            kind,
            possible: list,
            pruned: false,
        }
    }

    fn prune(&mut self, list: &[char]) {
        self.possible = self
            .possible
            .iter()
            .filter_map(|c| if list.contains(c) { Some(*c) } else { None })
            .collect();
        self.pruned = true;
    }

    pub fn select(
        &mut self,
        rng: &mut ThreadRng,
        consonants: &[Consonant],
        vowels: &[Vowel],
        nasal: &[char],
    ) {
        if self.pruned {
            let index = rng.gen_range(0..=self.entropy());
            let item = self.possible[index];
            self.possible = vec![item];
        } else {
            let proper_list = match self.kind {
                SyllableLetter::Consonant(_) => consonants,
                SyllableLetter::Vowel(_) => vowels,
                SyllableLetter::Nasal(_) => nasal,
            };
            self.prune(proper_list);
        }
    }

    pub fn get_letters(&self) -> Vec<char> {
        self.possible.clone()
    }

    pub fn entropy(&self) -> usize {
        self.possible.len() - 1
    }

    pub fn is_collapsed(&self) -> bool {
        self.entropy() == 0
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct SyllableOrder(pub Vec<SyllableLetter>);

impl SyllableOrder {
    pub fn insert(&mut self, index: usize, letter: SyllableLetter) {
        self.0.insert(index, letter);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyllableLetter {
    Consonant(u8),
    Vowel(u8),
    Nasal(u8),
}

impl SyllableLetter {
    pub fn probability(self) -> f64 {
        match self {
            SyllableLetter::Consonant(f) | SyllableLetter::Vowel(f) | SyllableLetter::Nasal(f) => {
                f64::from(f) / 100.0
            }
        }
    }

    pub fn change_probability(&mut self, p: u8) -> Self {
        match self {
            SyllableLetter::Consonant(f) | SyllableLetter::Vowel(f) | SyllableLetter::Nasal(f) => {
                *f = p;
            }
        }
        *self
    }
}

impl SyllableOrder {
    pub fn generate(
        &self,
        // rng: &mut ThreadRng,
        consonants: &[Consonant],
        vowels: &[Vowel],
        nasal: &[char],
    ) -> Syllable {
        // let mut syl = Syllable::new();

        // for letter in &self.0 {
        //     let list = if letter.is_consonant() {
        //         consonants
        //     } else if letter.is_vowel() {
        //         vowels
        //     } else {
        //         nasal
        //     };

        //     if list.is_empty() {
        //         continue;
        //     }

        //     let p = letter.probability();

        //     let c = if rng.gen_bool(p) {
        //         let len = list.len();
        //         let index = rng.gen_range(0..len);
        //         list[index]
        //     } else {
        //         continue;
        //     };

        //     syl.insert(syl.len(), c);
        // }

        // syl

        let mut gen = SyllableGenerator::new(
            self.clone(),
            consonants.to_vec(),
            vowels.to_vec(),
            nasal.to_vec(),
        );

        while !gen.is_collapsed() {
            // eprintln!("{:#?}", gen);
            gen.iterate();
        }

        gen.extract()
    }
}

impl FromStr for SyllableOrder {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut order = SyllableOrder::default();
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();

        let find_right = |chars: &[char], current: usize, find: char| -> Result<usize, Self::Err> {
            // if let Some(c) = chars.last() {
            //     if *c == find {
            //         return Ok(chars.len()-1);
            //     }
            // }

            for (i, c) in chars.iter().skip(current).enumerate() {
                if *c == find {
                    return Ok(current + i);
                }
            }

            Err("probability character has no end".into())
        };

        let mut prob = false;
        let mut next = None;
        for i in 0..len {
            if let Some(n) = next {
                if i < n {
                    continue;
                }
                next = None;
            }
            let p = if prob { 50 } else { 100 };
            let c = chars[i];
            match c {
                '(' => {
                    let mut end = find_right(&chars, i, ')')?;
                    next = Some(end);
                    let aa = &s[i + 1..end];
                    let comma = find_right(&aa.chars().collect::<Vec<char>>(), 0, ',').ok();
                    let probability = if let Some(comma) = comma {
                        end = comma;
                        aa[comma + 1..].parse::<u8>()?
                    } else {
                        end = aa.len();
                        50
                    };
                    let character =
                        aa[0..end].parse::<SyllableOrder>()?.0[0].change_probability(probability);
                    order.insert(order.len(), character);
                }
                ')' => {
                    prob = false;
                    continue;
                }
                'c' | 'C' => order.insert(order.len(), SyllableLetter::Consonant(p)),
                'v' | 'V' => order.insert(order.len(), SyllableLetter::Vowel(p)),
                'n' | 'N' => order.insert(order.len(), SyllableLetter::Nasal(p)),
                _ => {}
            }
        }

        Ok(order)
    }
}

#[test]
fn so_parse_normal() {
    let input = "cvc";
    let so: SyllableOrder = input.parse().unwrap();

    assert_eq!(
        so,
        SyllableOrder(vec![
            SyllableLetter::Consonant(100),
            SyllableLetter::Vowel(100),
            SyllableLetter::Consonant(100)
        ])
    );
}

#[test]
fn so_parse_normal_prob() {
    let input = "cv(c)";
    let so: SyllableOrder = input.parse().unwrap();

    assert_eq!(
        so,
        SyllableOrder(vec![
            SyllableLetter::Consonant(100),
            SyllableLetter::Vowel(100),
            SyllableLetter::Consonant(50)
        ])
    );
}

#[test]
fn so_parse_set_prob() {
    let input = "cv(c,25)";
    let so: SyllableOrder = input.parse().unwrap();

    assert_eq!(
        so,
        SyllableOrder(vec![
            SyllableLetter::Consonant(100),
            SyllableLetter::Vowel(100),
            SyllableLetter::Consonant(25)
        ])
    );
}

#[derive(Debug)]
struct Generator {
    pub consonants: Vec<Consonant>,
    pub vowels: Vec<Vowel>,
    pub nasal: Vec<char>,
    pub order: SyllableOrder,
    pub syllables: u128,
    pub count: u128,
}

impl Generator {
    pub fn generate(&self) -> Vec<Word> {
        let mut words: Vec<Word> = Vec::new();
        for _ in 0..self.count {
            let mut word: Word = Word::new();
            for _ in 0..self.syllables {
                word = format!(
                    "{}{}",
                    word,
                    self.order
                        .generate(&self.consonants, &self.vowels, &self.nasal)
                );
            }

            if !words.contains(&word) {
                words.insert(words.len(), word);
            }

            words.sort();
        }

        words
    }
}

fn main() -> Result<(), Error> {
    use jargon_args::Jargon;
    let mut j: Jargon = Jargon::from_env();

    if j.contains(["-h", "--help"]) {
        help();
    } else if j.contains(["-V", "--version"]) {
        version();
    } else {
        let moment = std::time::Instant::now();
        let gen: Generator = Generator {
            consonants: j
                .result_arg::<String, [&str; 2]>(["-c", "--consonants"])?
                .chars()
                .collect(),
            vowels: j
                .result_arg::<String, [&str; 2]>(["-v", "--vowels"])?
                .chars()
                .collect(),
            nasal: j
                .option_arg::<String, [&str; 2]>(["-n", "--nasals"])
                .unwrap_or_else(|| "".to_string())
                .chars()
                .collect(),
            order: j
                .result_arg::<String, [&str; 2]>(["-o", "--order"])?
                .parse()?,
            syllables: j
                .option_arg(["-s", "--syllables"])
                .unwrap_or_else(|| "2".to_string())
                .parse()?,
            count: j
                .option_arg(["-C", "--count"])
                .unwrap_or_else(|| "4".to_string())
                .parse()?,
        };

        let words = gen.generate();

        let mut oup = std::io::stdout();
        for word in words {
            oup.write_all(format!("{}\n", word).as_bytes())?;
        }
        oup.flush()?;

        eprintln!("took {} millisecond(s)", moment.elapsed().as_millis());
    }

    Ok(())
}

type Error = Box<dyn std::error::Error>;

fn help() {
    print!(
        "Rawrs - Random Annoying Words

\t-c, --consonants\tList of consonants to use
\t-h, --help\t\tView this help
\t-o, --order\t\tSelect syllable order
\t-s, --syllables\t\tNumber of syllables for each word
\t-v, --vowels\t\tList of vowels to use
\t-V, --version\t\tView version information\n"
    );
}

fn version() {
    println!("Rawrs version {}", env!("CARGO_PKG_VERSION"));
}
