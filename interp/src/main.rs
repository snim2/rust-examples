use std::boxed::Box;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::io::prelude::*;
use std::io;

#[derive(PartialEq, Eq, Hash)]
enum Associativity { LEFT, RIGHT, }

enum SyntaxError {
    UnknownSymbol(String),
    MismatchedParentheses,
    GeneralError,
}

#[derive(PartialEq, Eq, Hash)]
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
    println!("Calculator REPL. Type 'quit' or 'exit' to end session.");
    println!("Place spaces between all tokens: 1 + ( 2 * 3 )");
    while !terminated {
        let mut line = String::new();
        print!(">>> ");
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
                Err(errno) =>
                    match errno {
                        UnknownSymbol(symbol) => println!("Unknown symbol: {}", symbol),
                        MismatchedParentheses => println!("Mismatched ( and )."),
                        GeneralError => println!("Syntax error."),
                    },
           }
       }
    }
}


fn lex(line: &String) -> LinkedList<Token> {
    use Token::*;
    let strings = line.trim().split(" ");
    let mut tokens: LinkedList<Token> = LinkedList::new();
    for lexeme in strings {
        match lexeme {
            "^" => tokens.push_back(POW),
            "+" => tokens.push_back(PLUS),
            "-" => tokens.push_back(MINUS),
            "*" => tokens.push_back(TIMES),
            "/" => tokens.push_back(DIVIDE),
            "%" => tokens.push_back(MODULO),
            "(" => tokens.push_back(LPAREN),
            ")" => tokens.push_back(RPAREN),
            number => match number.parse() {
                        Ok(num) => tokens.push_back(LexicalNumber(num)),
                        Err(_) => tokens.push_back(LexicalError(String::from(lexeme))),
                      },
        }
    }
    tokens
}


fn parse(tokens: LinkedList<Token>) -> Result<Expr, SyntaxError> {
    use Associativity::*;
    use Expr::*;
    use SyntaxError::*;
    use Token::*;
    // Operator-precedence table.
    let mut op_table : HashMap<Token, (u32, Associativity)> = HashMap::new();
    op_table.insert(POW,    (4, RIGHT));
    op_table.insert(TIMES,  (3, LEFT));
    op_table.insert(DIVIDE, (3, LEFT));
    op_table.insert(PLUS,   (2, LEFT));
    op_table.insert(MINUS,  (2, LEFT));
    op_table.insert(MODULO, (1, LEFT));
    op_table.insert(LPAREN, (9, LEFT));
    op_table.insert(RPAREN, (0, LEFT));
    // Dijkstra's shunting-yard algorithm.
    let mut operator_stack: Vec<Token> = Vec::new();
    let mut operand_queue: LinkedList<Expr> = LinkedList::new();
    for token in tokens {
        match token {
            LexicalError(error) => return Err(UnknownSymbol(error.clone())),
            LexicalNumber(number) => operand_queue.push_front(Number(number)),
            LPAREN => operator_stack.push(LPAREN),
            RPAREN => {
                while *operator_stack.last().unwrap() != LPAREN {
                    if operator_stack.len() == 0 {
                        return Err(MismatchedParentheses);
                    }
                    let l_op = operand_queue.pop_front().unwrap();
                    let r_op = operand_queue.pop_front().unwrap();
                    match construct_expr(operator_stack.pop(), l_op, r_op) {
                        Ok(expr) => operand_queue.push_back(expr),
                        Err(error) => return Err(error),
                    };
                };
                if operator_stack.len() == 0 {
                    return Err(MismatchedParentheses);
                }
                operator_stack.pop();  // Remove matching LPAREN.
            },
            operator =>
                if operator_stack.len() == 0 ||
                   *operator_stack.last().unwrap() == LPAREN {
                      operator_stack.push(operator);
                } else {
                    loop {
                        let op2 = operator_stack.pop().unwrap();
                        let (ref p1, ref a1) = *(op_table.get(&operator).unwrap());
                        let (ref p2, _) = *(op_table.get(&op2).unwrap());
                        if (p1 < p2 && *a1 == RIGHT) || (p1 <= p2 && *a1 == LEFT) {
                            let l_op = operand_queue.pop_front().unwrap();
                            let r_op = operand_queue.pop_front().unwrap();
                            let op2_pop = operator_stack.pop();
                            match construct_expr(op2_pop, l_op, r_op) {
                                Ok(expr) => operand_queue.push_back(expr),
                                Err(error) => return Err(error),
                            }
                        } else {
                            operator_stack.push(op2);
                            break;
                        }
                    }
                    operator_stack.push(operator);
                },
        }
    }
    // All tokens have been consumed from user input.
    while operator_stack.len() > 0 {
        match operator_stack.pop() {
            None => return Err(GeneralError),
            Some(LPAREN) => return Err(MismatchedParentheses),
            Some(RPAREN) => return Err(MismatchedParentheses),
            Some(LexicalError(error)) => return Err(UnknownSymbol(error.clone())),
            operator => {
                if operand_queue.len() < 2 {
                    return Err(GeneralError);
                } else {
                    let l_op = operand_queue.pop_front().unwrap();
                    let r_op = operand_queue.pop_front().unwrap();
                    match construct_expr(operator, l_op, r_op) {
                        Ok(expr) => operand_queue.push_back(expr),
                        Err(error) => return Err(error),
                    };
                }
            },
        };
    };
    if operand_queue.len() != 1 {
        return Err(GeneralError);
    }
    Ok(operand_queue.pop_front().unwrap())
}


fn construct_expr(token: Option<Token>, l_op: Expr, r_op: Expr) -> Result<Expr, SyntaxError> {
    use Expr::*;
    use SyntaxError::*;
    use Token::*;
    let expr : Expr;
    match token {
        Some(POW) => expr = Pow(Box::new(l_op), Box::new(r_op)),
        Some(DIVIDE) => expr = Divide(Box::new(l_op), Box::new(r_op)),
        Some(TIMES) => expr = Times(Box::new(l_op), Box::new(r_op)),
        Some(PLUS) => expr = Plus(Box::new(l_op), Box::new(r_op)),
        Some(MINUS) => expr = Minus(Box::new(l_op), Box::new(r_op)),
        Some(MODULO) => expr = Modulo(Box::new(l_op), Box::new(r_op)),
        Some(LexicalError(error)) => return Err(UnknownSymbol(error.clone())),
        _ => return  Err(GeneralError),
    };
    Ok(expr)
}


fn evaluate(expr: &Expr) -> Result<i32, String> {
    use Expr::*;
    match expr {
        &Number(n) => Ok(n),
        &Pow(ref e_left, ref e_right) =>
            Ok(evaluate(e_left).unwrap().pow(evaluate(e_right).unwrap() as u32)),
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
