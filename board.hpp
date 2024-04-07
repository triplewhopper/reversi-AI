#pragma once

#include <iostream>
#include <bitset>
#include <cassert>

enum class Player {
    Black,
    White
};

std::ostream& operator<<(std::ostream& os, Player player) {
    return os << (player == Player::Black ? "Black" : "White");
}

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
    return os << "Pos::" << names[63 - std::underlying_type_t<Pos>(pos)];
}

typedef uint64_t bitboard_t;

constexpr bitboard_t one_hot(Pos pos) {
    return 1ULL << static_cast<std::underlying_type_t<Pos>>(pos);
}

constexpr int test_bit(bitboard_t bb, Pos pos) {
    return (bb & (1ULL << static_cast<std::underlying_type_t<Pos>>(pos))) != 0;
}

template <size_t i>
constexpr bitboard_t shift(bitboard_t bb) {
    static_assert(i < 8);
    switch (i) {
        case 0: return bb >> 1;
        case 1: return bb >> 9;
        case 2: return bb >> 8;
        case 3: return bb >> 7;
        case 4: return bb << 1;
        case 5: return bb << 9;
        case 6: return bb << 8;
        case 7: return bb << 7;
    }
}
std::string bitboard_to_string(bitboard_t bb) {
    std::string res = "  ABCDEFGH\n";
    for (int i = 0; i < 8; i++) {
        res += std::to_string(i + 1) + ' ';
        for (int j = 0; j < 8; j++) {
            auto b = test_bit(bb, static_cast<Pos>(63 - (i * 8 + j)));
            res += b? "*": " ";
        }
        res += "\n";
    }
    return res;
}

class Board {
    bitboard_t black, white;
    constexpr Board(bitboard_t black, bitboard_t white): black(black), white(white) {
        if (black & white) throw std::invalid_argument("black & white != 0");
    }
    constexpr bitboard_t get_disk(Player p) const {
        return p == Player::Black ? black : white;
    }
public:
    constexpr Board(): black(0x0000'0008'1000'0000), white(0x0000'0010'0800'0000) {}
    constexpr bool is_final() const {
        return !valid_moves(Player::Black) && !valid_moves(Player::White);
    }

    constexpr int count(Player p) const { return __builtin_popcountll(get_disk(p)); }

    constexpr bool operator==(Board const& other) const { return black == other.black && white == other.white; }

    constexpr bitboard_t valid_moves(Player p) const {
        auto const empty = ~(black | white);
        uint64_t moves = 0;
        auto const me = p == Player::Black ? black : white;
        auto const opp = p == Player::Black ? white : black;
        auto const o1 = opp & 0x7e7e'7e7e'7e7e'7e7eULL;

#define S(o, i) \
do { \
    auto x = o & shift<i>(me); \
    x |= o & shift<i>(x); \
    x |= o & shift<i>(x); \
    x |= o & shift<i>(x); \
    x |= o & shift<i>(x); \
    x |= o & shift<i>(x); \
    moves |= shift<i>(x); \
} while (0)

        S(o1, 0);
        S(o1, 1);
        S(opp, 2);
        S(o1, 3);
        S(o1, 4);
        S(o1, 5);
        S(opp, 6);
        S(o1, 7);
#undef S
        return moves & empty;
    }
    
    constexpr Board place_at(Player p, Pos pos) const {
        auto const me = (p == Player::Black ? black : white) | one_hot(pos);
        auto const opp = p == Player::Black ? white : black;
        auto const o1 = opp & 0x7e7e'7e7e'7e7e'7e7eULL;
        uint64_t captured_discs = 0;

#define S(o, i) \
do { \
    auto x = o & shift<i>(one_hot(pos)); \
    x |= o & shift<i>(x); \
    x |= o & shift<i>(x); \
    x |= o & shift<i>(x); \
    x |= o & shift<i>(x); \
    x |= o & shift<i>(x); \
    x &= - static_cast<int64_t>((me & shift<i>(x)) != 0); \
    captured_discs |= x; \
} while (0)

        S(o1, 0);
        S(o1, 1);
        S(opp, 2);
        S(o1, 3);
        S(o1, 4);
        S(o1, 5);
        S(opp, 6);
        S(o1, 7);
#undef S
        switch (p) {
            case Player::Black: return Board(me ^ captured_discs, opp ^ captured_discs);
            case Player::White: return Board(opp ^ captured_discs, me ^ captured_discs);
        }
    }
    friend std::ostream& operator<<(std::ostream& os, Board const& board);
};

std::ostream& operator<<(std::ostream& os, Board const& board) {
    os << "  ABCDEFGH\n";
    for (int i = 0; i < 8; i++) {
        os << i + 1 << ' ';
        for (int j = 0; j < 8; j++) {
            auto b = test_bit(board.get_disk(Player::Black), static_cast<Pos>(63 - (i * 8 + j)));
            auto w = test_bit(board.get_disk(Player::White), static_cast<Pos>(63 - (i * 8 + j)));
            assert(b + w <= 1);
            os << (b? "●": w? "○": " ");
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

