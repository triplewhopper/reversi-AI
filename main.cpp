#include <iostream>
#include "board.hpp"

int main() {
    constexpr auto b = Board();
    std::cout << b << "\n";
    auto b1 = b.place_at(Player::Black, Pos::C4);
    std::cout << b1 << "\n";
    b1 = b1.place_at(Player::White, Pos::C5);
    std::cout << b1 << "\n";
    return 0;
}