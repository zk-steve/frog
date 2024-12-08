# Architecture

## Context

The goal is to implement an MPC (Multi-Party Computation) protocol where two clients each send an encrypted 64-bit
integer (`u64`) to a server. The server computes the sum of the two encrypted values and returns the result to the
clients. The clients can then decrypt the result collaboratively.

## Overview

![Overview](attachments/overview.svg)

### Common Configurations

Before the protocol starts, some common configurations are required for all services, including:

- Parameters for using the Phantom library.
- A `crs_seed` to generate the **common reference string (CRS)**.

### Simplified Flow

1. **Parameter Exchange Between Clients and Server**:
    - Clients send the ring packing key and public key to the server.
    - The server aggregates these keys into a final public key and returns it to the clients.
    - Clients generate bootstrap keys and send them to the server.
    - The server aggregates the bootstrap keys and stores the parameters in its database.

2. **Data Encryption and Submission**:
    - Each client encrypts a `u64` value and sends it to the server.
    - The server adds a task to its database, signaling workers to compute the result in the encrypted domain.

3. **Computation by Workers**:
    - Workers pick up pending tasks, perform the computation, and save the encrypted result back to the server's
      database.

4. **Result Retrieval**:
    - Clients query the server to retrieve the encrypted result.

5. **Decryption**:
    - Clients exchange decryption shares of the result.
    - Clients aggregate these shares to collaboratively decrypt and obtain the final result.