use colored::{ColoredString, Colorize};

fn main() {
    let banner: ColoredString = "d8888b. db       .d8b.  d888888b d88888b d88888b  .d88b.  d8888b. .88b  d88. d88888b      .88b  d88.  .d8b.  d8888b. d888888b d888888b d888888b .88b  d88. d88888b 
88  `8D 88      d8' `8b `~~88~~' 88'     88'     .8P  Y8. 88  `8D 88'YbdP`88 88'          88'YbdP`88 d8' `8b 88  `8D   `88'   `~~88~~'   `88'   88'YbdP`88 88'     
88oodD' 88      88ooo88    88    88ooooo 88ooo   88    88 88oobY' 88  88  88 88ooooo      88  88  88 88ooo88 88oobY'    88       88       88    88  88  88 88ooooo 
88~~~   88      88~~~88    88    88~~~~~ 88~~~   88    88 88`8b   88  88  88 88~~~~~      88  88  88 88~~~88 88`8b      88       88       88    88  88  88 88~~~~~ 
88      88booo. 88   88    88    88.     88      `8b  d8' 88 `88. 88  88  88 88.          88  88  88 88   88 88 `88.   .88.      88      .88.   88  88  88 88.     
88      Y88888P YP   YP    YP    Y88888P YP       `Y88P'  88   YD YP  YP  YP Y88888P      YP  YP  YP YP   YP 88   YD Y888888P    YP    Y888888P YP  YP  YP Y88888P".red().bold().blink();

    println!("{banner}");
}
