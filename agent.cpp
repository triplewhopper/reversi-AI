#include "agent.hpp"
#include "mcts.hpp"
#include <memory>

void RandomAgent::initialize() & {}
void RandomAgent::opponent_move_callback(const Action *) & {}
std::unique_ptr<Action> RandomAgent::select_move(Board const& board, Player p) & {
    auto moves = board.valid_moves(p);
    dist = std::uniform_int_distribution<int>(0, __builtin_popcountll(moves) - 1);
    for (auto i = dist(engine); i; i--) {
        moves -= moves & (~moves + 1);
    }
    assert(moves > 0);
    auto move = static_cast<Pos>(63 - __builtin_clzll(moves));
    return std::make_unique<Action>(move, board.place_at(p, move));
}

template <Player P>
void MCTSAgent<P>::initialize() & {
    tree = std::move(std::make_unique<MCTS<Player::Black>>(GameState<Player::Black>{Board(), 0}));
    if constexpr (P == Player::White) {
        cursor = nullptr;
    } else {
        cursor = tree->root.get();
    }
    n_simulations = 1000;
    exploration = 1.0;
}
