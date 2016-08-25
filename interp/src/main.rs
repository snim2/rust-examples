use std::boxed::Box;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io;


enum SyntaxError {
    UnknownSymbol(String),
    MissingRParen,
    MissingLParen,
    GeneralError,
}


#[derive(Debug, PartialEq, Eq, Hash)]
enum Token {
    LexicalError(String), LexicalNumber(i32), POW, PLUS, MINUS, TIMES, DIVIDE,
    MODULO, LPAREN, RPAREN,
}


enum Expr {
    Number(i32),
    Plus(Box<Expr>, Box<Expr>),
    Minus(Box<Expr>, Box<Expr>),
    Times(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
    Modulo(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
}


fn main() {
    use SyntaxError::*;
    let mut expression: Result<Expr, SyntaxError>;
    let mut terminated = false;
    while !terminated {
        let mut line = String::new();
        print!("> ");
        io::stdout().flush().ok().expect("Failed to flush from STDOUT.");
        io::stdin().read_line(&mut line).ok().expect("Failed to read from STDIN.");
        line = String::from(line.trim());
        if line == "exit" || line == "quit" {
            terminated = true;
        } else if line == "" {
            continue
        } else {
            expression = parse(lex(&line));
            match expression {
                Ok(expr) => match evaluate(&expr) {
                                Ok(number) => println!("{}", number),
                                Err(_) => println!("Cannot divide by zero!")
                            },
                Err(errno) => match errno {
                                  UnknownSymbol(symbol) => println!("Unknown symbol: {}", symbol),
                                  MissingRParen => println!("Missing )."),
                                  MissingLParen => println!("Missing (."),
                                  GeneralError => println!("Syntax error."),
                              },
           }
       }
    }
}


fn lex(line: &String) -> Vec<Token> {
    use Token::*;
    let strings = line.trim().split(" ");
    let mut tokens: Vec<Token> = Vec::new();
    for lexeme in strings {
        match lexeme {
            "^" => tokens.push(POW),
            "+" => tokens.push(PLUS),
            "-" => tokens.push(MINUS),
            "*" => tokens.push(TIMES),
            "/" => tokens.push(DIVIDE),
            "%" => tokens.push(MODULO),
            "(" => tokens.push(LPAREN),
            ")" => tokens.push(RPAREN),
            number => match number.parse() {
                        Ok(num) => tokens.push(LexicalNumber(num)),
                        Err(_) => tokens.push(LexicalError(String::from(lexeme))),
                      }
        }
    }
    tokens
}


fn parse(tokens: Vec<Token>) -> Result<Expr, SyntaxError> {
    use Expr::*;
    use SyntaxError::*;
    use Token::*;
    // Operator-precedence table.
    let mut op_table : HashMap<Token, (u32, &str)> = HashMap::new();
    op_table.insert(POW,    (4, "R"));
    op_table.insert(TIMES,  (3, "L"));
    op_table.insert(DIVIDE, (3, "L"));
    op_table.insert(PLUS,   (2, "L"));
    op_table.insert(MINUS,  (2, "L"));
    op_table.insert(MODULO, (1, "L"));
    op_table.insert(LPAREN, (9, "L"));
    op_table.insert(RPAREN, (0, "L"));
    // Dijkstra's shunting-yard algorithm.
    let mut operator_stack: Vec<Token> = Vec::new();
    let mut operand_stack: Vec<Expr> = Vec::new();
    'outer: for token in tokens {
        match token {
            LexicalError(error) => return Err(UnknownSymbol(error.clone())),
            LexicalNumber(number) => operand_stack.push(Number(number)),
            LPAREN => operator_stack.push(LPAREN),
            RPAREN => (),
            _ => (),
            // POW => (),
            // DIVIDE => (),
            // TIMES => (),
            // PLUS => (),
            // MINUS => (),
            // MODULO => (),
        }
    }
    if operand_stack.len() == 1 {
        let expr: Expr = operand_stack.pop().unwrap();
        return Ok(expr);
    }
    Err(GeneralError)
}


fn evaluate(expr: &Expr) -> Result<i32, String> {
    use Expr::*;
    match expr {
        &Number(n) => Ok(n),
        &Pow(ref e_left, ref e_right) =>
            Ok(evaluate(e_left).unwrap() ^ evaluate(e_right).unwrap()),
        &Plus(ref e_left, ref e_right) =>
            Ok(evaluate(e_left).unwrap() + evaluate(e_right).unwrap()),
        &Minus(ref e_left, ref e_right) =>
            Ok(evaluate(e_left).unwrap() - evaluate(e_right).unwrap()),
        &Times(ref e_left, ref e_right) =>
            Ok(evaluate(e_left).unwrap() * evaluate(e_right).unwrap()),
        &Divide(ref e_left, ref e_right) => {
            let result: i32 = evaluate(e_right).unwrap();
            if result == 0 {
                return Err(String::from("Division by zero!"));
            } else {
                return Ok(evaluate(e_left).unwrap() / result);
            }
        },
        &Modulo(ref e_left, ref e_right) => {
            let result: i32 = evaluate(e_right).unwrap();
            if result == 0 {
                return Err(String::from("Division by zero!"));
            } else {
                return Ok(evaluate(e_left).unwrap() % result);
            }
        },
    }
}
