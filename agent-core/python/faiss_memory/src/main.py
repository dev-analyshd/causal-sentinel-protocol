#!/usr/bin/env python3
"""
FAISS Behavioral Memory

Stores 128-dimensional behavioral vectors for agents and provides
similarity search for the self-reflection plane.
"""

import os
import pickle
from typing import List, Tuple, Optional
from dataclasses import dataclass

import numpy as np
import faiss
import structlog

logger = structlog.get_logger()


@dataclass
class MemoryConfig:
    vector_dim: int = 128
    index_type: str = "IVF_FLAT"  # or "Flat", "HNSW"
    nlist: int = 100  # Number of clusters for IVF
    max_vectors: int = 1_000_000
    index_path: str = "./faiss_index/behavioral.index"
    metadata_path: str = "./faiss_index/metadata.pkl"


class FAISSBehavioralMemory:
    """FAISS-based behavioral memory for agent self-reflection."""

    def __init__(self, config: MemoryConfig = None):
        self.config = config or MemoryConfig()
        self.index = None
        self.metadata: List[dict] = []
        self.vector_count = 0

        self._init_index()

    def _init_index(self):
        """Initialize FAISS index."""
        if os.path.exists(self.config.index_path):
            self.index = faiss.read_index(self.config.index_path)
            with open(self.config.metadata_path, "rb") as f:
                self.metadata = pickle.load(f)
            self.vector_count = len(self.metadata)
            logger.info("index_loaded", vectors=self.vector_count)
        else:
            os.makedirs(os.path.dirname(self.config.index_path), exist_ok=True)

            if self.config.index_type == "IVF_FLAT":
                quantizer = faiss.IndexFlatIP(self.config.vector_dim)
                self.index = faiss.IndexIVFFlat(
                    quantizer, self.config.vector_dim, self.config.nlist, faiss.METRIC_INNER_PRODUCT
                )
                # Need training data for IVF
                training = np.random.randn(1000, self.config.vector_dim).astype("float32")
                self.index.train(training)
            else:
                self.index = faiss.IndexFlatIP(self.config.vector_dim)

            logger.info("index_created", type=self.config.index_type, dim=self.config.vector_dim)

    def store(self, agent_id: str, vector: np.ndarray, timestamp: float, block: int) -> int:
        """Store a behavioral vector."""
        if vector.shape[0] != self.config.vector_dim:
            raise ValueError(f"Expected dim {self.config.vector_dim}, got {vector.shape[0]}")

        # Normalize for cosine similarity
        vector = vector.astype("float32")
        vector = vector / np.linalg.norm(vector)

        # Add to index
        self.index.add(vector.reshape(1, -1))

        # Store metadata
        meta = {
            "id": self.vector_count,
            "agent_id": agent_id,
            "timestamp": timestamp,
            "block": block,
        }
        self.metadata.append(meta)
        self.vector_count += 1

        # Save periodically
        if self.vector_count % 1000 == 0:
            self.save()

        logger.debug("vector_stored", agent_id=agent_id, id=meta["id"])
        return meta["id"]

    def search(self, query: np.ndarray, k: int = 10) -> List[Tuple[int, float, dict]]:
        """Search for k nearest neighbors."""
        query = query.astype("float32")
        query = query / np.linalg.norm(query)

        distances, indices = self.index.search(query.reshape(1, -1), k)

        results = []
        for idx, dist in zip(indices[0], distances[0]):
            if idx >= 0 and idx < len(self.metadata):
                results.append((idx, float(dist), self.metadata[idx]))

        return results

    def search_by_agent(self, agent_id: str, query: np.ndarray, k: int = 10) -> List[Tuple[int, float, dict]]:
        """Search within a specific agent's history."""
        results = self.search(query, k=k*2)
        return [r for r in results if r[2]["agent_id"] == agent_id][:k]

    def get_agent_history(self, agent_id: str) -> List[dict]:
        """Get all stored vectors for an agent."""
        return [m for m in self.metadata if m["agent_id"] == agent_id]

    def compute_memory_density(self, agent_id: str, recent_vector: np.ndarray) -> float:
        """Compute memory density (average similarity to historical vectors)."""
        history = self.get_agent_history(agent_id)

        if len(history) < 10:
            return 0.5

        # Search for similar vectors
        results = self.search_by_agent(agent_id, recent_vector, k=min(100, len(history)))

        if not results:
            return 0.5

        avg_similarity = np.mean([r[1] for r in results])
        return (avg_similarity + 1.0) / 2.0  # Normalize to [0, 1]

    def save(self):
        """Save index and metadata to disk."""
        faiss.write_index(self.index, self.config.index_path)
        with open(self.config.metadata_path, "wb") as f:
            pickle.dump(self.metadata, f)
        logger.info("index_saved", vectors=self.vector_count)

    def __del__(self):
        """Cleanup."""
        if self.index is not None:
            self.save()


if __name__ == "__main__":
    # Demo
    memory = FAISSBehavioralMemory()

    # Store some vectors
    for i in range(100):
        vec = np.random.randn(128)
        memory.store(f"agent_{i % 3}", vec, float(i), i)

    # Search
    query = np.random.randn(128)
    results = memory.search(query, k=5)
    print(f"Top 5 neighbors: {results}")
