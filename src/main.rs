use rand::{Rng, thread_rng};
use std::str::FromStr;
use rand::rngs::ThreadRng;

type Word = String;
type Words = Vec<Word>;
type Consonant = char;
type Vowel = char;
type Syllable = String;

#[derive(Debug)]
struct Consonants(Vec<Consonant>);
#[derive(Debug)]
struct Vowels(Vec<Vowel>);

#[derive(Debug)]
enum SyllableOrder {
    Vc,
    Cv,
    Cvv,
    Cvc,
    Ccv,
    Vcv,
    Vcc,
    Vvc,
}

impl FromStr for SyllableOrder {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_uppercase().as_str() {
            "VC" => Self::Vc,
            "CV" => Self::Cv,
            "CVV" => Self::Cvv,
            "CVC" => Self::Cvc,
            "CCV" => Self::Ccv,
            "VCV" => Self::Vcv,
            "VCC" => Self::Vcc,
            "VVC" => Self::Vvc,
            _ => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "failed to parse syllable order",
                )))
            }
        })
    }
}

#[derive(Debug)]
struct Generator {
    pub consonants: Vec<Consonant>,
    pub vowels: Vec<Vowel>,
    pub order: SyllableOrder,
    pub syllables: u128,
    pub count: u128,
}

impl Generator {
    pub fn generate(&self, rng: &mut ThreadRng) -> Words {
        let mut words: Words = Words::new();

        for _ in 0..self.count {
            let mut word: Word = Word::new();
            for _ in 0..self.syllables {
                word = format!(
                    "{}{}",
                    word,
                    match self.order {
                        SyllableOrder::Vc => self.vc(rng),
                        SyllableOrder::Cv => self.cv(rng),
                        SyllableOrder::Cvv => self.cvv(rng),
                        SyllableOrder::Cvc => self.cvc(rng),
                        SyllableOrder::Ccv => self.ccv(rng),
                        SyllableOrder::Vcv => self.vcv(rng),
                        SyllableOrder::Vcc => self.vcc(rng),
                        SyllableOrder::Vvc => self.vvc(rng),
                    }
                )
            }
            words.append(&mut vec![word])
        }

        words
    }

    fn vc(&self, rng: &mut ThreadRng) -> Syllable {
        if rng.gen() && self.consonants.contains(&'m') && self.consonants.contains(&'n') {
            format!("{}{}", self.v(rng), self.n(rng))
        } else {
            format!("{}{}", self.v(rng), self.c(rng))
        }
    }

    fn cv(&self, rng: &mut ThreadRng) -> Syllable {
        format!("{}{}", self.c(rng), self.v(rng))
    }

    fn cvv(&self, rng: &mut ThreadRng) -> Syllable {
        if rng.gen() {
            format!("{}{}", self.cv(rng), self.v(rng))
        } else {
            self.cv(rng)
        }
    }

    fn cvc(&self, rng: &mut ThreadRng) -> Syllable {
        if rng.gen() {
            format!("{}{}", self.cv(rng), self.c(rng))
        } else {
            if rng.gen() && self.consonants.contains(&'m')  && self.consonants.contains(&'n') {
                format!("{}{}", self.cv(rng), self.n(rng))
            } else {
                self.cv(rng)
            }
        }
    }

    fn ccv(&self, rng: &mut ThreadRng) -> Syllable {
        if rng.gen() {
            format!("{}{}", self.c(rng), self.cv(rng))
        } else {
            self.cv(rng)
        }
    }

    fn vcv(&self, rng: &mut ThreadRng) -> Syllable {
        if rng.gen() {
            format!("{}{}", self.vc(rng), self.v(rng))
        } else {
            self.vc(rng)
        }
    }

    fn vcc(&self, rng: &mut ThreadRng) -> Syllable {
        if rng.gen() {
            format!("{}{}", self.vc(rng), self.c(rng))
        } else {
            if rng.gen() && self.consonants.contains(&'m')  && self.consonants.contains(&'n') {
                format!("{}{}", self.vc(rng), self.n(rng))
            } else {
                self.vc(rng)
            }
        }
    }

    fn vvc(&self, rng: &mut ThreadRng) -> Syllable {
        if rng.gen() {
            format!("{}{}", self.v(rng), self.vc(rng))
        } else {
            if rng.gen() && self.consonants.contains(&'m')  && self.consonants.contains(&'n') {
                format!("{}{}{}", self.v(rng), self.v(rng), self.n(rng))
            } else {
                self.vc(rng)
            }
        }
    }

    fn c(&self, rng: &mut ThreadRng) -> Consonant {
        let idx: usize = rng.gen_range(0..self.consonants.len());
        self.consonants[idx]
    }

    fn v(&self, rng: &mut ThreadRng) -> Vowel {
        let idx: usize = rng.gen_range(0..self.vowels.len());
        self.vowels[idx]
    }

    fn n(&self, rng: &mut ThreadRng) -> Consonant {
        if rng.gen() {
            'm'
        } else {
            'n'
        }
    }
}

fn main() -> Result<(), Error> {
    use jargon_args::Jargon;
    let mut rng: ThreadRng = thread_rng();
    let mut j: Jargon = Jargon::from_env();

    if j.contains(["-h", "--help"]) {
        help();
    } else if j.contains(["-V", "--version"]) {
        version();
    } else {
        let gen: Generator = Generator {
            consonants: j.result_arg(["-c", "--consonants"])?.chars().collect(),
            vowels: j.result_arg(["-v", "--vowels"])?.chars().collect(),
            order: j.result_arg(["-o", "--order"])?.parse()?,
            syllables: j
                .option_arg(["-s", "--syllables"])
                .unwrap_or_else(|| "2".to_string())
                .parse()?,
            count: j
                .option_arg(["-c", "--count"])
                .unwrap_or_else(|| "4".to_string())
                .parse()?,
        };

        let words: Words = gen.generate(&mut rng);

        for word in words.iter() {
            println!("{}", word);
        }
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
\t-V, --version\t\tView version information\n");
}

fn version() {
    println!("Rawrs version {}", env!("CARGO_PKG_VERSION"));
}