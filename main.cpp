#include <iostream>
#include "board.hpp"

int main() {
    auto b = Board();
    std::cout << b << "\n";
    std::string input;
    Player p = Player::Black;
    while (true) {
        std::cout << "valid moves for player " << p << ":\n" << bitboard_to_string(b.valid_moves(p)) << "\n";
        if (!(std::cin >> input)) break;
        assert (input.size() == 2);
        assert (input[0] >= 'A' && input[0] <= 'H' || input[0] >= 'a' && input[0] <= 'h');
        assert (input[1] >= '1' && input[1] <= '8');
        input[0] = toupper(input[0]);
        auto pos = static_cast<Pos>(7 - (input[0] - 'A') + (7 - (input[1] - '1')) * 8);
        std::cout << "player " << p << " moves on " << pos << "\n";
        try {
            b = b.place_at(p, pos);
        } catch (std::exception const& e) {
            std::cerr << "invalid move: " << e.what() << "\n";
            continue;
        }
        p = !p;
        std::cout << b << "\n";
    }
    return 0;
}