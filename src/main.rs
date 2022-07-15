#![forbid(unsafe_code)]
#![forbid(unstable_features)]
#![forbid(missing_fragment_specifier)]
#![warn(clippy::all, clippy::pedantic)]
use async_std::task;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

// mod file;

type Word = String;
type Consonant = String;
type Vowel = String;
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
    nasal: Vec<String>,
    syllable: Vec<LetterGenerator>,
    seeded: bool,
    index: usize,
}

impl SyllableGenerator {
    pub fn new(
        order: SyllableOrder,
        consonants: Vec<Consonant>,
        vowels: Vec<Vowel>,
        nasal: Vec<String>,
    ) -> SyllableGenerator {
        Self {
            order,
            consonants,
            vowels,
            nasal,
            syllable: Vec::new(),
            seeded: false,
            index: 0,
        }
    }

    pub async fn iterate(&mut self, rng: &mut ThreadRng) {
        if self.syllable.is_empty() {
            self.create_syllable(rng).await;
        }

        self.seed(rng).await;

        if self.is_collapsed() {
            return;
        }

        if self.index == self.syllable.len() {
            self.index = 0;
        }

        self.syllable[self.index]
            .select(rng, &self.consonants, &self.vowels, &self.nasal)
            .await;

        self.index += 1;
    }

    async fn list(&self) -> Vec<String> {
        let mut list = Vec::new();

        list.append(&mut self.consonants.clone());
        list.append(&mut self.vowels.clone());
        list.append(&mut self.nasal.clone());

        list
    }

    pub async fn extract(&mut self) -> Syllable {
        let mut chars: Vec<String> = Vec::new();

        for letter in &self.syllable {
            let mut letters = letter.get_letters();
            chars.append(&mut letters);
        }

        chars.iter().map(|c| c.to_string()).collect()
    }

    async fn seed(&mut self, rng: &mut ThreadRng) {
        if self.seeded {
            return;
        }

        if let Some(start) = self.get_random_uncollapsed(rng).await {
            self.syllable[start]
                .select(rng, &self.consonants, &self.vowels, &self.nasal)
                .await;
            self.seeded = true;
        }
    }

    async fn get_random_uncollapsed(&mut self, rng: &mut ThreadRng) -> Option<usize> {
        if self.is_collapsed() {
            return None;
        }

        let len = self.syllable.len();
        loop {
            let index = rng.gen_range(0..len);

            if self.syllable[index].is_collapsed() {
                continue;
            }

            return Some(index);
        }
    }

    async fn create_syllable(&mut self, rng: &mut ThreadRng) {
        let mut syl: Vec<LetterGenerator> = Vec::new();

        for s in &self.order.0 {
            if rng.gen_bool(s.probability()) {
                syl.insert(syl.len(), LetterGenerator::new(*s, self.list().await));
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
    possible: Vec<String>,
    pruned: bool,
}

impl LetterGenerator {
    pub fn new(kind: SyllableLetter, list: Vec<String>) -> Self {
        Self {
            kind,
            possible: list,
            pruned: false,
        }
    }

    async fn prune(&mut self, list: &[String]) {
        self.possible = self
            .possible
            .iter()
            .filter_map(|c| {
                if list.contains(c) {
                    Some(c.to_owned())
                } else {
                    None
                }
            })
            .collect();
        self.pruned = true;
    }

    pub async fn select(
        &mut self,
        rng: &mut ThreadRng,
        consonants: &[Consonant],
        vowels: &[Vowel],
        nasal: &[String],
    ) {
        if self.pruned {
            let index = rng.gen_range(0..=self.entropy());
            let item = self.possible[index].clone();
            self.possible = vec![item];
        } else {
            let proper_list = match self.kind {
                SyllableLetter::Consonant(_) => consonants,
                SyllableLetter::Vowel(_) => vowels,
                SyllableLetter::Nasal(_) => nasal,
            };
            self.prune(proper_list).await;
        }
    }

    pub fn get_letters(&self) -> Vec<String> {
        self.possible.clone()
    }

    pub fn entropy(&self) -> usize {
        if self.possible.len() == 0 {
            panic!("possible outcomes cannot be 0");
        }
        (self.possible.len()).checked_sub(1).unwrap_or(0)
    }

    pub fn is_collapsed(&self) -> bool {
        self.entropy() == 0
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct SyllableOrder(pub Vec<SyllableLetter>);

impl SyllableOrder {
    pub fn insert(&mut self, index: usize, letter: SyllableLetter) {
        self.0.insert(index, letter);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
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
    // #[async_recursion]
    pub async fn generate(
        &self,
        // rng: &mut ThreadRng,
        consonants: &[Consonant],
        vowels: &[Vowel],
        nasal: &[String],
    ) -> Option<String> {
        let mut rng = rand::thread_rng();
        let mut gen = SyllableGenerator::new(
            self.clone(),
            consonants.to_vec(),
            vowels.to_vec(),
            nasal.to_vec(),
        );

        while !gen.is_collapsed() {
            gen.iterate(&mut rng).await;
        }

        let output = gen.extract().await.trim().to_string();

        if output.is_empty() {
            None
        } else {
            Some(output)
        }
    }
}

impl FromStr for SyllableOrder {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut order = SyllableOrder::default();
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();

        let find_right = |chars: &[char], current: usize, find: char| -> Result<usize, Self::Err> {
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
    pub nasal: Vec<String>,
    pub order: SyllableOrder,
    pub syllables: Option<usize>,
    pub count: usize,
    sort: bool,
}

impl Generator {
    pub async fn generate(&self) -> Vec<Word> {
        if !self.verify() {
            return vec![];
        }
        let mut words: Vec<Word> = Vec::with_capacity(self.count);
        let mut i = words.len();
        let now = Instant::now();
        while i < self.count {
            let mut word: Word = Word::new();
            let syl = self
                .syllables
                .unwrap_or_else(|| rand::random::<usize>() % 4);
            let mut runs = 0;
            while runs < syl {
                match self
                    .order
                    .generate(&self.consonants, &self.vowels, &self.nasal)
                    .await
                {
                    Some(syllable) => word = format!("{}{}", word, syllable),
                    None => continue,
                }
                runs += 1;
            }
            word = word.trim().to_owned();
            if !words.contains(&word) && !word.is_empty() {
                words.insert(words.len(), word);
            }

            if self.sort {
                words.sort();
            }

            if now.elapsed().as_secs_f32() / 50.0 >= 1.0 {
                return words;
            }

            i = words.len();
        }

        words
    }

    fn verify(&self) -> bool {
        for l in &self.order.0 {
            let b = match l {
                SyllableLetter::Consonant(_) => self.consonants.len() > 1,
                SyllableLetter::Vowel(_) => self.vowels.len() > 1,
                SyllableLetter::Nasal(_) => self.nasal.len() > 1,
            };

            if !b {
                return b;
            }
        }

        true
    }
}

fn main() -> Result<(), Error> {
    use jargon_args::Jargon;
    let mut j: Jargon = Jargon::from_env();

    if j.contains(["-h", "--help"]) {
        help();
    } else if j.contains(["-V", "--version"]) {
        version();
    } else if let Some(file) = j.option_arg::<PathBuf, [&str; 2]>(["-f", "--file"]) {
        let mut file = File::open(file)?;
        let mut buf: String = String::new();
        file.read_to_string(&mut buf)?;

        #[derive(Debug, serde::Deserialize)]
        struct Tmp {
            Consonants: Vec<String>,
            Vowels: Vec<String>,
            Nasals: Vec<String>,
            Order: String,
        };

        let t: Tmp = toml::from_str(&buf)?;

        let gen: Generator = Generator {
            consonants: t.Consonants,
            vowels: t.Vowels,
            nasal: t.Nasals,
            order: t.Order.parse()?,
            syllables: j.option_arg::<usize, [&str; 2]>(["-s", "--syllables"]),
            count: j
                .option_arg(["-C", "--count"])
                .unwrap_or_else(|| "4".to_string())
                .parse()?,
            sort: j.contains(["-S", "--sort"]),
        };

        let moment = std::time::Instant::now();
        let words = task::block_on(gen.generate());
        let now = moment.elapsed().as_millis();
        let count = words.len();
        let mut oup = std::io::stdout();
        for word in words {
            oup.write_all(format!("{}\n", word).as_bytes())?;
        }
        oup.flush()?;

        eprintln!("made {} in {} millisecond(s)", count, now);
    } else {
        let gen: Generator = Generator {
            consonants: j
                .result_arg::<String, [&str; 2]>(["-c", "--consonants"])?
                .chars()
                .map(|s| s.to_string())
                .collect(),
            vowels: j
                .result_arg::<String, [&str; 2]>(["-v", "--vowels"])?
                .chars()
                .map(|s| s.to_string())
                .collect(),
            nasal: j
                .option_arg::<String, [&str; 2]>(["-n", "--nasals"])
                .unwrap_or_else(|| "".to_string())
                .chars()
                .map(|s| s.to_string())
                .collect(),
            order: j
                .result_arg::<String, [&str; 2]>(["-o", "--order"])?
                .parse()?,
            syllables: j.option_arg::<usize, [&str; 2]>(["-s", "--syllables"]),
            count: j
                .option_arg(["-C", "--count"])
                .unwrap_or_else(|| "4".to_string())
                .parse()?,
            sort: j.contains(["-S", "--sort"]),
        };

        let moment = std::time::Instant::now();
        let words = task::block_on(gen.generate());
        let now = moment.elapsed().as_millis();
        let count = words.len();
        let mut oup = std::io::stdout();
        for word in words {
            oup.write_all(format!("{}\n", word).as_bytes())?;
        }
        oup.flush()?;

        eprintln!("made {} in {} millisecond(s)", count, now);
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
