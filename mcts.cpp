#include "mcts.hpp"

template <Player P>
inline Reward MCTSNode<P>::rollout() const {
    auto board = state.board;
    auto n_steps = state.n_steps;
    bool passed = false;
    Player player = P;
    for (;;) {
        if (auto action = player == P ? MCTSNode<P>::rollout_policy(state) : MCTSNode<!P>::rollout_policy(state)) {
            passed = false;
            board = action->board;
        } else {
            if (passed) break;
            passed = true;
        }
        player = !player;
        n_steps++;
    }

    switch (board.count(P) <=> board.count(!P)) {
        case std::strong_ordering::greater:
            return Reward::Win;
        case std::strong_ordering::less:
            return Reward::Loss;
        case std::strong_ordering::equal:
            return Reward::Draw;
    }
}

template<Player P>
std::unique_ptr<Action> 
MCTSNode<P>::rollout_policy(GameState<P> const& state) {
    auto moves = state.board.valid_moves(P);
    if (moves == 0) {
        return nullptr;
    }
    std::random_device seed_gen;
    std::mt19937 engine(seed_gen());
    std::uniform_int_distribution<int> dist(0, __builtin_popcountll(moves) - 1);
    for (auto i = dist(engine); i; i--) {
        moves -= moves & (~moves + 1);
    }
    assert(moves > 0);
    auto const move = static_cast<Pos>(63 - __builtin_clzll(moves));
    return std::make_unique<Action>(move, state.board.place_at(P, move));
}

template <Player P>
MCTSNode<!P> *
MCTSNode<P>::has_been_expanded_on(std::unique_ptr<Action> action) const{
    return std::find_if(children.begin(), children.end(), [&](auto const& node) {
        return node.causing_action == action;
    });
}