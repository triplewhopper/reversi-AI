#pragma once

#include <iostream>
#include <memory>
#include <random>
#include "board.hpp"

class Agent {
    public:
    virtual void initialize() & = 0;
    virtual void opponent_move_callback(const Action *) & = 0;
    virtual std::unique_ptr<Action> select_move(Board const&, Player) & = 0;
};

class RandomAgent : public Agent {
    std::random_device seed_gen;
    std::mt19937 engine;
    std::uniform_int_distribution<int> dist;

public:
    RandomAgent(): seed_gen(), engine(seed_gen()), dist(0, 63) {}
    void initialize() & override; 
    void opponent_move_callback(const Action *) & override; 
    std::unique_ptr<Action> select_move(Board const&, Player) & override;
};

template <Player P> class MCTSNode;
template <Player P> class MCTS;

template <Player P>
class MCTSAgent : public Agent {
    std::unique_ptr<MCTS<Player::Black>> tree;
    MCTSNode<P> * cursor;
    uint64_t n_simulations;
    double exploration; 

public:
    void initialize() & override;
    void opponent_move_callback(const Action *) & override;
    std::unique_ptr<Action> select_move(Board const&, Player) & override;
};