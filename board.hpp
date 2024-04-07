#pragma once

#include <iostream>
#include <bitset>
#include <cassert>

enum class Player {
    Black,
    White
};

[[nodiscard]]
constexpr inline Player operator!(Player player) {
    return player == Player::Black ? Player::White : Player::Black;
}

enum class Pos: uint8_t {
    A1 = 63, B1 = 62, C1 = 61, D1 = 60, E1 = 59, F1 = 58, G1 = 57, H1 = 56,
    A2 = 55, B2 = 54, C2 = 53, D2 = 52, E2 = 51, F2 = 50, G2 = 49, H2 = 48,
    A3 = 47, B3 = 46, C3 = 45, D3 = 44, E3 = 43, F3 = 42, G3 = 41, H3 = 40,
    A4 = 39, B4 = 38, C4 = 37, D4 = 36, E4 = 35, F4 = 34, G4 = 33, H4 = 32,
    A5 = 31, B5 = 30, C5 = 29, D5 = 28, E5 = 27, F5 = 26, G5 = 25, H5 = 24,
    A6 = 23, B6 = 22, C6 = 21, D6 = 20, E6 = 19, F6 = 18, G6 = 17, H6 = 16,
    A7 = 15, B7 = 14, C7 = 13, D7 = 12, E7 = 11, F7 = 10, G7 = 9, H7 = 8,
    A8 = 7, B8 = 6, C8 = 5, D8 = 4, E8 = 3, F8 = 2, G8 = 1, H8 = 0
};

std::ostream& operator<<(std::ostream& os, Pos pos) {
    static const char *names[] = {
        "A1", "B1", "C1", "D1", "E1", "F1", "G1", "H1",
        "A2", "B2", "C2", "D2", "E2", "F2", "G2", "H2",
        "A3", "B3", "C3", "D3", "E3", "F3", "G3", "H3",
        "A4", "B4", "C4", "D4", "E4", "F4", "G4", "H4",
        "A5", "B5", "C5", "D5", "E5", "F5", "G5", "H5",
        "A6", "B6", "C6", "D6", "E6", "F6", "G6", "H6",
        "A7", "B7", "C7", "D7", "E7", "F7", "G7", "H7",
        "A8", "B8", "C8", "D8", "E8", "F8", "G8", "H8"
    };
    return os << names[63 - static_cast<int>(pos)];
}

constexpr uint64_t MASK = 1;
constexpr uint64_t one_hot(Pos pos) {
    return MASK << static_cast<uint64_t>(pos);
}

class Board {
    uint64_t black, white;
    constexpr Board(uint64_t black, uint64_t white): black(black), white(white) {
        if (black & white) {
            throw std::invalid_argument("black & white != 0");
        }
    }
public:
    constexpr Board(): black(0x0000'0008'1000'0000), white(0x0000'0010'0800'0000) {}
    
    constexpr uint64_t get_disk(Player p) const {
        return p == Player::Black ? black : white;
    }

    constexpr bool is_final() const {
        return !valid_moves(Player::Black) && !valid_moves(Player::White);
    }

    constexpr uint32_t count(Player p) const {
        return p == Player::Black ? __builtin_popcountll(black) : __builtin_popcountll(white);
    }

    constexpr void flip_discs(uint64_t discs) & {
        black ^= discs;
        white ^= discs;
    }
    constexpr void swap_discs() & {
        std::swap(black, white);
    }

    constexpr bool operator==(Board const& other) const {
        return black == other.black && white == other.white;
    }

    static constexpr uint64_t shift(uint64_t x, size_t i) {
        constexpr std::tuple<int, uint64_t> shifts[] = {
            {1, 0x7f7f'7f7f'7f7f'7f7f},
            {9, 0x007f'7f7f'7f7f'7f7f},
            {8, 0xffff'ffff'ffff'ffff},
            {7, 0x00fe'fefe'fefe'fefe},
            {-1, 0xfefe'fefe'fefe'fefe},
            {-9, 0xfefe'fefe'fefe'fe00},
            {-8, 0x7f7f'7f7f'7f7f'7f7f},
            {-7, 0x7f7f'7f7f'7f7f'7f00},
        };
        auto &[shift, mask] = shifts[i];
        if (i < 4) {
            return (x >> shift) & mask;
        } else {
            return (x << -shift) & mask;
        }
    }
    
    constexpr uint64_t valid_moves(Player p) const {
        uint64_t const empty = ~(black | white);
        uint64_t moves = 0;
        auto const me = p == Player::Black ? black : white;
        auto const opp = p == Player::Black ? white : black;
        for (size_t i = 0; i < 8; i++) {
            auto x = shift(me, i) & opp;
            for (size_t j = 0; j < 5; j++) {
                x |= shift(x, i) & opp;
            }
            moves |= shift(x, i) & empty;
        }
        return moves;
    }
    constexpr Board place_at(Player p, Pos pos) const {
        auto const me = (p == Player::Black ? black : white) | one_hot(pos);
        auto const opp = p == Player::Black ? white : black;
        uint64_t captured_discs = 0;
        for (size_t i = 0; i < 8; i++) {
            auto x = shift(one_hot(pos), i) & opp;
            for (size_t j = 0; j < 5; j++) {
                x |= shift(x, i) & opp;
            }
            if (shift(x, i) & me) {
                captured_discs |= x;
            }
        }
        auto res = Board(me, opp);
        res.flip_discs(captured_discs);
        switch (p) {
            case Player::Black:
                return res;
            case Player::White:
                res.swap_discs();
                return res;
        }
    }
};

std::ostream& operator<<(std::ostream& os, Board const& board) {
    os << "  ABCDEFGH\n";
    for (int i = 0; i < 8; i++) {
        os << i + 1 << ' ';
        for (int j = 0; j < 8; j++) {
            auto b = board.get_disk(Player::Black) >> (63 - (i * 8 + j)) & 1;
            auto w = board.get_disk(Player::White) >> (63 - (i * 8 + j)) & 1;
            assert(b + w <= 1);
            if (b) {
                os << "●";
            } else if (w) {
                os << "○";
            } else {
                os << " ";
            }
        }
        os << "\n";
    }
    return os;
}

class Action {
    Pos pos;
    Board board;
    public:
    constexpr Action(Pos pos, Board board): pos(pos), board(board) {}
    constexpr bool operator==(Action const& other) const {
        return pos == other.pos && board == other.board;
    }
};