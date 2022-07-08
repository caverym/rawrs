#![forbid(unsafe_code)]
#![forbid(unstable_features)]
#![forbid(missing_fragment_specifier)]
#![warn(clippy::all, clippy::pedantic)]
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
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

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct SyllableOrder(Vec<SyllableLetter>);

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
    pub fn is_consonant(self) -> bool {
        matches!(self, SyllableLetter::Consonant(_))
    }

    pub fn is_vowel(self) -> bool {
        matches!(self, SyllableLetter::Vowel(_))
    }

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
                *f = p
            }
        }
        *self
    }
}

impl SyllableOrder {
    pub fn generate(
        &self,
        rng: &mut ThreadRng,
        consonants: &[Consonant],
        vowels: &[Vowel],
        nasal: &[char],
    ) -> Syllable {
        let mut syl = Syllable::new();

        for letter in &self.0 {
            let list = if letter.is_consonant() {
                consonants
            } else if letter.is_vowel() {
                vowels
            } else {
                nasal
            };

            if list.is_empty() {
                continue;
            }

            let p = letter.probability();

            let c = if rng.gen_bool(p) {
                let len = list.len();
                let index = rng.gen_range(0..len);
                list[index]
            } else {
                continue;
            };

            syl.insert(syl.len(), c);
        }

        syl
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
        let mut rng: ThreadRng = thread_rng();

        for _ in 0..self.count {
            let mut word: Word = Word::new();
            for _ in 0..self.syllables {
                word = format!(
                    "{}{}",
                    word,
                    self.order
                        .generate(&mut rng, &self.consonants, &self.vowels, &self.nasal)
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
