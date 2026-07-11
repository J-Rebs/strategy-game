# PacketCommand

A modern, abstract, blocky real-time strategy (RTS) game written in **Rust** using the **Bevy** engine. 

In **PacketCommand**, you build and secure a network grid, making strategic topology and resource allocation trade-offs to out-compete and dominate the network against a Computer AI opponent.

---

## 🎮 Gameplay & Strategic Trade-offs

The game is grounded in real computer networking concepts. You build connections, configure routing, lay down security policies, and manage server queue congestion:

1. **Network Topologies**:
   - **Star**: Lay a fast central Switch. Cheap to link, low latency, but a single point of failure.
   - **Mesh**: Connect nodes redundantly. Highly resilient to cuts and DDoS, but expensive.
   - **Ring/Bus**: Linear chain of nodes. Very cheap to build, but high latency and easily partitioned.

2. **Physical Links**:
   - **Fiber Optic**: Maximum bandwidth, near-instant propagation, high cost.
   - **Copper**: Moderate bandwidth, higher latency, cheap to lay.
   - **Wireless**: Instant deployment (no cables), moderate bandwidth, high packet loss / error rates.

3. **Resource Nodes (Competitive Hubs)**:
   - **Data Centers (DCs)**: Generate passive Bandwidth currency when processing Client requests.
   - **Internet Exchanges (IXPs)**: Boost performance of connected links (2x speed multiplier).
   - **Content Delivery Networks (CDNs)**: Cache data locally, reducing RTT (Round Trip Time).

4. **Win Condition**:
   - Dominate the network by capturing the neutral Data Centers and either compromising the opponent's Gateway or partitioning their network to starve them of bandwidth currency.

---

## 🛠️ Development & Compilation

### Requirements
- **Rust (v1.80+)**
- **macOS ARM (Apple Silicon)** or any platform supported by Bevy.

### Run Locally
To compile and run the desktop application natively:
```bash
cargo run
```

### Dev Optimizations
To keep incremental compilation times under 3 seconds, the project is configured to use Bevy's `dynamic_linking` feature and high dependency optimization overrides in `Cargo.toml`.

---

## 📚 Developer Documentation

We provide a beautiful, custom developer documentation website inside the `/docs` folder that explains the Rust architecture, Bevy ECS systems, borrowing panic solutions, and hexagonal math.

### Spin Up Documentation Locally
You can spin up the documentation site locally using any simple HTTP server:

**Using Python:**
```bash
python3 -m http.server --directory docs 8080
```
Then open [http://localhost:8080](http://localhost:8080) in your browser.

**Using Node.js:**
```bash
npx -y serve docs
```

**Alternative:**
Simply double-click the [docs/index.html](file:///Users/joe/Documents/projects/strategy-game/docs/index.html) file to open it directly in any web browser.

### Deploy to GitHub Pages
To publish the documentation site to GitHub Pages:
1. Go to your repository settings on GitHub.
2. Select **Pages** in the left sidebar.
3. Under **Build and deployment -> Source**, select **Deploy from a branch**.
4. Set the branch to `main` and choose the `/docs` folder.
5. Click **Save**. GitHub will automatically publish the documentation site.

