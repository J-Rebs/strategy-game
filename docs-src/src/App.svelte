<script>
  let activeTab = $state('intro');

  function setTab(tab) {
    activeTab = tab;
  }

  const ecsCode = `// System that moves packets along their associated links
fn move_packets(
    mut commands: Commands,
    links: Query<&NetworkLink>,
    mut packets: Query<(Entity, &mut Packet)>,
) {
    for (packet_entity, mut packet) in packets.iter_mut() {
        if let Ok(link) = links.get(packet.link) {
            packet.progress += link.link_type.speed() * 0.01;
            // ...
        }
    }
}`;

  const hexCode = `// Rounding fractional coordinates (q, r) to the nearest integer HexCoord
fn round(q: f32, r: f32) -> HexCoord {
    let s = -q - r;
    let mut rq = q.round();
    let mut rr = r.round();
    let rs = s.round();

    let dq = (rq - q).abs();
    let dr = (rr - r).abs();
    let ds = (rs - s).abs();

    if dq > dr && dq > ds {
        rq = -rr - rs;
    } else if dr > ds {
        rr = -rq - rs;
    }

    HexCoord { q: rq as i32, r: rr as i32 }
}`;
</script>

<div class="glow-backdrop"></div>

<div class="container">
  <!-- Sidebar Navigation -->
  <aside class="sidebar">
    <div class="logo-area">
      <!-- Custom SVG illustration of sea wave and hexagonal packet -->
      <svg class="logo-svg" viewBox="0 0 100 60" width="80" height="48">
        <path d="M10,40 Q25,25 40,40 T70,40 T100,40" fill="none" stroke="#9FD3D3" stroke-width="3" />
        <polygon points="50,12 65,22 65,38 50,48 35,38 35,22" fill="#EBF9F9" stroke="#9FD3D3" stroke-width="2" />
        <circle cx="50" cy="30" r="4" fill="#ffb85c" />
      </svg>
      <h1>PacketCommand</h1>
      <span class="sub">Developer Hub</span>
    </div>

    <nav class="nav-links">
      <button 
        class="nav-btn {activeTab === 'intro' ? 'active' : ''}" 
        onclick={() => setTab('intro')}
      >
        <span class="btn-icon">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
            <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
          </svg>
        </span>
        Introduction
      </button>

      <button 
        class="nav-btn {activeTab === 'ecs' ? 'active' : ''}" 
        onclick={() => setTab('ecs')}
      >
        <span class="btn-icon">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="7" height="7" rx="1" />
            <rect x="14" y="3" width="7" height="7" rx="1" />
            <rect x="14" y="14" width="7" height="7" rx="1" />
            <rect x="3" y="14" width="7" height="7" rx="1" />
          </svg>
        </span>
        Bevy ECS Engine
      </button>

      <button 
        class="nav-btn {activeTab === 'rust' ? 'active' : ''}" 
        onclick={() => setTab('rust')}
      >
        <span class="btn-icon">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <path d="M12 8v4l3 3" />
          </svg>
        </span>
        Rust Core Concepts
      </button>

      <button 
        class="nav-btn {activeTab === 'hex' ? 'active' : ''}" 
        onclick={() => setTab('hex')}
      >
        <span class="btn-icon">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 2l8.66 5v10L12 22l-8.66-5V7z" stroke-linejoin="round" />
          </svg>
        </span>
        Hexagonal Math
      </button>

      <button 
        class="nav-btn {activeTab === 'network' ? 'active' : ''}" 
        onclick={() => setTab('network')}
      >
        <span class="btn-icon">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="22" y1="12" x2="2" y2="12" />
            <path d="M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" />
            <line x1="6" y1="16" x2="6.01" y2="16" />
            <line x1="10" y1="16" x2="10.01" y2="16" />
          </svg>
        </span>
        OSPF Network Sim
      </button>
    </nav>

    <div class="sidebar-footer">
      <a href="https://github.com/J-Rebs/strategy-game" target="_blank" rel="noopener noreferrer" class="repo-link">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" class="github-icon">
          <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
        </svg>
        GitHub Repository
      </a>
      <p>Docs built with Svelte</p>
      <p>Game built with Rust</p>
      <p class="coordinator">Orchestrated with Antigravity</p>
    </div>
  </aside>

  <!-- Main Content Area -->
  <main class="content-area">
    {#if activeTab === 'intro'}
      <section class="doc-section">
        <h2>Introduction</h2>
        <p class="subtitle">Undersea Network Strategy Game Architecture</p>
        
        <!-- Custom Pastel WIP Notice Warning Panel (No emojis) -->
        <div class="card warning-card">
          <div class="card-header">
            <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#E1A140" stroke-width="2" class="alert-icon">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
              <line x1="12" y1="9" x2="12" y2="13" />
              <line x1="12" y1="17" x2="12.01" y2="17" />
            </svg>
            <h3>Active Alpha Stage</h3>
          </div>
          <p>The basic game compiles and runs, but mechanics are still under development. This developer documentation is currently AI-generated and requires review.</p>
        </div>

        <p class="paragraph">Welcome to the developer documentation hub for PacketCommand, an undersea real-time network strategy simulation game. The player builds nodes, lays cables, and competes for central Data Centers in a vibrant, bioluminescent coral reef setting.</p>
        
        <div class="card">
          <div class="card-header">
            <!-- Custom SVG drawing representing a target/philosophy -->
            <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#9FD3D3" stroke-width="2" class="header-icon">
              <circle cx="12" cy="12" r="10" />
              <circle cx="12" cy="12" r="6" />
              <circle cx="12" cy="12" r="2" />
            </svg>
            <h3>Project Philosophy and Goals</h3>
          </div>
          <p>The primary purpose of PacketCommand is to help contributors learn how to effectively write code and learn in collaboration with AI. The overarching goal is to help engineers learn how to move quicker while constantly improving their own core understanding. We take as our guide the continuous, iterative improvement of the engineer, the engineer's use of AI tools, and the game itself. New contributors and feedback are highly welcome!</p>
        </div>
        
        <div class="card">
          <div class="card-header">
            <!-- Custom SVG drawing representing a light bulb / concept -->
            <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#9FD3D3" stroke-width="2" class="header-icon">
              <circle cx="12" cy="12" r="10" />
              <path d="M8 14s1.5 2 4 2 4-2 4-2" />
              <line x1="9" y1="9" x2="9.01" y2="9" />
              <line x1="15" y1="9" x2="15.01" y2="9" />
            </svg>
            <h3>Game Concept and Goal</h3>
          </div>
          <p>The core gameplay maps real-world computer networking concepts onto a pointy-topped 3D hexagonal map. Players configure routers, upgrade queue processing speed, set up firewalls, and deploy worm payloads to capture adjacent nodes in real-time, working towards network dominance.</p>
        </div>

        <!-- Custom SVG Wave drawing at bottom of intro -->
        <div class="wave-footer">
          <svg viewBox="0 0 100 20" width="100%" height="30" preserveAspectRatio="none">
            <path d="M0,10 Q25,0 50,10 T100,10 L100,20 L0,20 Z" fill="#EBF9F9" />
          </svg>
        </div>
      </section>
    {/if}

    {#if activeTab === 'ecs'}
      <section class="doc-section">
        <h2>Bevy ECS Architecture</h2>
        <p class="subtitle">Entity Component System Decoupling</p>
        <p class="paragraph">Bevy is a modern, data-driven game engine written in Rust. Game logic is split using the ECS paradigm, separating state from system processing functions:</p>
        
        <div class="grid-2">
          <div class="card">
            <div class="card-header">
              <!-- Custom Cube Drawing -->
              <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#9FD3D3" stroke-width="2" class="header-icon">
                <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
                <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
                <line x1="12" y1="22.08" x2="12" y2="12" />
              </svg>
              <h3>Entities and Components</h3>
            </div>
            <p>Entities are simple ID handles representing nodes, links, and packets. Components are plain Rust structs containing data (such as NetworkNode, NetworkLink, Packet) attached to these IDs.</p>
          </div>

          <div class="card">
            <div class="card-header">
              <!-- Custom Gear/System Drawing -->
              <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#9FD3D3" stroke-width="2" class="header-icon">
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
              </svg>
              <h3>Systems</h3>
            </div>
            <p>Systems are standard Rust functions that run every frame. They query for specific components and mutate state (e.g., move_packets, update_routing_tables, camera_controls).</p>
          </div>
        </div>

        <div class="card code-card">
          <div class="card-header">
            <h3>System Query Example</h3>
          </div>
          <pre><code>{ecsCode}</code></pre>
        </div>
      </section>
    {/if}

    {#if activeTab === 'rust'}
      <section class="doc-section">
        <h2>Rust Core Concepts</h2>
        <p class="subtitle">Safety, Concurrency, and Aliasing Rules</p>
        <p class="paragraph">Rust guarantees memory safety and concurrency correctness at compile-time. Here are the core concepts used in PacketCommand:</p>

        <div class="card">
          <div class="card-header">
            <!-- Custom Rust Crab Outline (No emoji) -->
            <svg viewBox="0 0 100 50" width="80" height="40" fill="none" stroke="#E1A140" stroke-width="2" class="rust-crab-svg">
              <ellipse cx="50" cy="25" rx="16" ry="12" fill="#FFEAA7" />
              <!-- Legs -->
              <path d="M30,25 Q15,20 10,12 M30,28 Q15,30 12,38 M30,31 Q15,40 16,48" />
              <path d="M70,25 Q85,20 90,12 M70,28 Q85,30 88,38 M70,31 Q85,40 84,48" />
              <!-- Eyes -->
              <circle cx="44" cy="18" r="2" fill="#000" />
              <circle cx="56" cy="18" r="2" fill="#000" />
              <!-- Pincers -->
              <path d="M40,15 Q30,5 24,10" />
              <path d="M60,15 Q70,5 76,10" />
            </svg>
            <h3>Ownership and Borrowing (System Conflict B0002)</h3>
          </div>
          <p>Rust enforces strict borrowing rules: you can have **many shared references** (&T) OR **exactly one mutable reference** (&mut T) to a resource, but never both at the same time.</p>
          <p><strong>Bevy Runtime Panic B0002</strong> occurred when our system parameters declared both <code>Res&lt;GameResources&gt;</code> and <code>ResMut&lt;GameResources&gt;</code>. Because Bevy runs systems in parallel, this violated Rust's aliasing rules at runtime. The fix consolidated all reads and writes into a single mutable <code>ResMut&lt;GameResources&gt;</code> access parameter.</p>
        </div>

        <div class="grid-2">
          <div class="card">
            <div class="card-header">
              <!-- Pattern matching target diagram -->
              <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#9FD3D3" stroke-width="2" class="header-icon">
                <circle cx="12" cy="12" r="10" />
                <circle cx="12" cy="12" r="6" />
                <circle cx="12" cy="12" r="2" fill="#9FD3D3" />
              </svg>
              <h3>Pattern Matching</h3>
            </div>
            <p>Used to handle packet types, link types, and owner checks safely and exhaustively. Evaluated at compile-time to guarantee no unhandled conditions.</p>
          </div>

          <div class="card">
            <div class="card-header">
              <!-- Local variables tag -->
              <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#9FD3D3" stroke-width="2" class="header-icon">
                <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" />
                <line x1="7" y1="7" x2="7.01" y2="7" stroke-width="3" />
              </svg>
              <h3>Local State Variables</h3>
            </div>
            <p>Egui locals (e.g., <code>Local&lt;u32&gt;</code>) are used to persist state (like IP sequence allocations and packet IDs) between frames without leaking variables globally.</p>
          </div>
        </div>
      </section>
    {/if}

    {#if activeTab === 'hex'}
      <section class="doc-section">
        <h2>Hexagonal Grid Mathematics</h2>
        <p class="subtitle">Pointy-Topped Axial Coordinates & World Projections</p>
        <p class="paragraph">The Catan-style board uses a Pointy-Topped Hexagonal Grid with radius 3 (19 hexes total). We map coordinate centers using axial coordinates (q, r):</p>

        <div class="grid-2">
          <div class="card">
            <div class="card-header">
              <!-- Hex grid projection math -->
              <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#9FD3D3" stroke-width="2" class="header-icon">
                <polygon points="12,2 22,7 22,17 12,22 2,17 2,7" stroke-linejoin="round" />
                <line x1="12" y1="2" x2="12" y2="22" />
                <line x1="2" y1="12" x2="22" y2="12" />
              </svg>
              <h3>Hex to 3D World Project</h3>
            </div>
            <p>For a hex radius R, the 3D position (x, z) of coordinate (q, r) is computed as:</p>
            <p><code>x = R * (sqrt(3) * q + sqrt(3)/2 * r)</code></p>
            <p><code>z = R * (3/2 * r)</code></p>
          </div>

          <div class="card">
            <div class="card-header">
              <!-- Mouse cursor picking -->
              <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#9FD3D3" stroke-width="2" class="header-icon">
                <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z" />
              </svg>
              <h3>3D Ray-to-Hex Picking</h3>
            </div>
            <p>Clicking translates screen space to a 3D ray. We intersect it with the y = 0 plane, convert (x, z) back to fractional hex coordinates, and round it to the nearest hex coordinate using distance comparison.</p>
          </div>
        </div>

        <div class="card code-card">
          <div class="card-header">
            <h3>Hex Coordinates Rounding Algorithm</h3>
          </div>
          <pre><code>{hexCode}</code></pre>
        </div>
      </section>
    {/if}

    {#if activeTab === 'network'}
      <section class="doc-section">
        <h2>OSPF Network & Routing Simulation</h2>
        <p class="subtitle">Shortest-Path Forwarding and Queue Management</p>
        
        <div class="card">
          <div class="card-header">
            <!-- Dijkstra Routing map graph -->
            <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#9FD3D3" stroke-width="2" class="header-icon">
              <circle cx="6" cy="18" r="3" fill="#EBF9F9" />
              <circle cx="18" cy="6" r="3" fill="#EBF9F9" />
              <circle cx="18" cy="18" r="3" fill="#EBF9F9" />
              <line x1="9" y1="18" x2="15" y2="18" />
              <line x1="18" y1="9" x2="18" y2="15" />
              <line x1="8" y1="16" x2="16" y2="8" />
            </svg>
            <h3>Shortest-Path OSPF Routing (Dijkstra)</h3>
          </div>
          <p>PacketCommand runs a dynamic shortest-path routing algorithm modeled after real-world **OSPF (Open Shortest Path First)**. Whenever links are laid or destroyed, the pathfinder executes a Dijkstra algorithm, updating every node's routing table dynamically.</p>
        </div>

        <div class="grid-2">
          <div class="card">
            <div class="card-header">
              <!-- Buffer icon -->
              <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#9FD3D3" stroke-width="2" class="header-icon">
                <line x1="3" y1="6" x2="21" y2="6" />
                <line x1="3" y1="12" x2="21" y2="12" />
                <line x1="3" y1="18" x2="21" y2="18" />
                <circle cx="6" cy="6" r="1.5" fill="#9FD3D3" />
                <circle cx="12" cy="12" r="1.5" fill="#9FD3D3" />
                <circle cx="18" cy="18" r="1.5" fill="#ffb85c" />
              </svg>
              <h3>Queue Buffers and Congestion</h3>
            </div>
            <p>Nodes have limited buffer capacities (e.g. 15 packets). If packets arrive faster than the node's CPU can process them, the queue congests. Overflowing packets are discarded, simulating packet drop rates.</p>
          </div>

          <div class="card">
            <div class="card-header">
              <!-- Worm malware code warning -->
              <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#ffb85c" stroke-width="2" class="header-icon">
                <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
              </svg>
              <h3>Malware Infection and Capture</h3>
            </div>
            <p>Worm packets traveling along connections decrement a target node's health. When health drops to 0, ownership shifts (e.g. Neutral to Player/AI), capturing the hex and redirecting routing tables.</p>
          </div>
        </div>
      </section>
    {/if}

    <footer class="mobile-footer">
      <a href="https://github.com/J-Rebs/strategy-game" target="_blank" rel="noopener noreferrer" class="repo-link">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" class="github-icon">
          <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
        </svg>
        GitHub Repository
      </a>
      <p>Docs built with Svelte • Game built with Rust</p>
      <p class="coordinator">Orchestrated with Antigravity</p>
    </footer>
  </main>
</div>

<style>
  /* Local stylesheet overrides and styling hooks */
  :global(:root) {
    --bg-dark: #F0F7F7;
    --bg-panel: rgba(255, 255, 255, 0.85);
    --border-color: rgba(159, 211, 211, 0.5);
    --border-glow: rgba(159, 211, 211, 0.9);
    --aqua: #7FB8B8;
    --coral-gold: #E1A140;
    --text-primary: #34495E;
    --text-secondary: #5D6D7E;
    --code-bg: #EAEFF2;
  }

  :global(body) {
    font-family: 'Outfit', sans-serif;
    background-color: var(--bg-dark);
    color: var(--text-primary);
    line-height: 1.6;
    margin: 0;
    padding: 0;
    height: 100vh;
    overflow: hidden;
  }

  .glow-backdrop {
    position: fixed;
    top: -20%;
    left: -20%;
    width: 140%;
    height: 140%;
    background: radial-gradient(circle at 80% 20%, rgba(159, 211, 211, 0.25) 0%, transparent 60%),
                radial-gradient(circle at 10% 90%, rgba(255, 197, 197, 0.2) 0%, transparent 50%);
    z-index: -1;
    pointer-events: none;
  }

  .container {
    display: grid;
    grid-template-columns: 280px 1fr;
    height: 100vh;
    overflow: hidden;
  }

  .sidebar {
    background-color: rgba(255, 255, 255, 0.9);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    padding: 2.5rem 1.75rem;
    z-index: 10;
  }

  .logo-area {
    margin-bottom: 3.5rem;
    text-align: center;
  }

  .logo-svg {
    display: block;
    margin: 0 auto 1rem;
  }

  .logo-area h1 {
    font-size: 1.45rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .logo-area .sub {
    font-size: 0.8rem;
    color: var(--text-secondary);
    letter-spacing: 1px;
    text-transform: uppercase;
    font-weight: 600;
  }

  .nav-links {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
    flex-grow: 1;
  }

  .nav-btn {
    background: none;
    border: 1px solid transparent;
    color: var(--text-secondary);
    padding: 0.8rem 1.2rem;
    text-align: left;
    font-size: 0.95rem;
    font-family: inherit;
    font-weight: 600;
    border-radius: 8px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .btn-icon {
    display: flex;
    align-items: center;
    color: var(--text-secondary);
    transition: color 0.3s;
  }

  .nav-btn:hover {
    color: var(--text-primary);
    background-color: rgba(159, 211, 211, 0.15);
    border-color: rgba(159, 211, 211, 0.2);
    transform: translateX(4px);
  }

  .nav-btn:hover .btn-icon {
    color: var(--text-primary);
  }

  .nav-btn.active {
    color: #fff;
    background-color: var(--aqua);
    border-color: var(--aqua);
    box-shadow: 0 4px 15px rgba(127, 184, 184, 0.35);
  }

  .nav-btn.active .btn-icon {
    color: #fff;
  }

  .sidebar-footer {
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-align: center;
    border-top: 1px solid var(--border-color);
    padding-top: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .sidebar-footer .repo-link {
    color: var(--text-primary);
    text-decoration: none;
    font-weight: 600;
    margin-bottom: 0.5rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
    font-size: 0.8rem;
    transition: color 0.2s;
  }

  .sidebar-footer .repo-link:hover {
    color: var(--aqua);
  }

  .github-icon {
    flex-shrink: 0;
  }

  .sidebar-footer .coordinator {
    font-weight: 600;
    color: var(--aqua);
    margin-top: 0.25rem;
  }

  .content-area {
    padding: 4rem 5rem;
    overflow-y: auto;
    height: 100vh;
  }

  .doc-section {
    max-width: 800px;
    margin: 0 auto;
    animation: fadeIn 0.4s ease-out forwards;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  h2 {
    font-size: 2.25rem;
    font-weight: 700;
    margin: 0 0 0.5rem;
    color: var(--text-primary);
  }

  .subtitle {
    font-size: 1.05rem;
    color: var(--aqua);
    margin-bottom: 2.5rem;
    text-transform: uppercase;
    letter-spacing: 1px;
    font-weight: 700;
  }

  .paragraph {
    font-size: 1.05rem;
    color: var(--text-secondary);
    margin-bottom: 2rem;
  }

  .card {
    background: var(--bg-panel);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    padding: 2rem;
    margin-bottom: 2rem;
    box-shadow: 0 8px 24px rgba(159, 211, 211, 0.08);
    transition: all 0.3s ease;
  }

  .card:hover {
    border-color: var(--border-glow);
    box-shadow: 0 8px 28px rgba(159, 211, 211, 0.15);
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .card-header h3 {
    font-size: 1.25rem;
    margin: 0;
    color: var(--text-primary);
    font-weight: 600;
  }

  .header-icon {
    flex-shrink: 0;
    color: var(--aqua);
  }

  .card p {
    font-size: 1rem;
    color: var(--text-secondary);
    margin: 0;
  }

  .warning-card {
    border-left: 4px solid var(--coral-gold);
    background-color: rgba(255, 235, 200, 0.3);
  }

  .warning-card h3 {
    color: #A0722C;
  }

  .alert-icon {
    color: var(--coral-gold);
  }

  .rust-crab-svg {
    display: block;
    margin-bottom: 1rem;
  }

  .grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
  }

  .code-card {
    padding: 0;
    overflow: hidden;
  }

  .code-card .card-header {
    padding: 1.25rem 2rem;
    border-bottom: 1px solid var(--border-color);
    margin-bottom: 0;
    background-color: rgba(159, 211, 211, 0.05);
  }

  pre {
    margin: 0;
    padding: 2rem;
    background-color: var(--code-bg);
    overflow-x: auto;
  }

  code {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.9rem;
    color: var(--text-primary);
  }

  .wave-footer {
    margin-top: 4rem;
  }

  .mobile-footer {
    display: none;
  }

  @media (max-width: 950px) {
    .container {
      grid-template-columns: 1fr;
      grid-template-rows: auto 1fr;
    }

    .sidebar {
      padding: 1.5rem;
      border-right: none;
      border-bottom: 1px solid var(--border-color);
    }

    .sidebar .sidebar-footer {
      display: none;
    }

    .logo-area {
      margin-bottom: 1.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 1rem;
    }

    .logo-svg {
      margin: 0;
    }

    .nav-links {
      flex-direction: row;
      overflow-x: auto;
      padding-bottom: 0.5rem;
    }

    .nav-btn {
      white-space: nowrap;
    }

    .content-area {
      padding: 2rem;
    }

    .grid-2 {
      grid-template-columns: 1fr;
    }

    .mobile-footer {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 0.25rem;
      margin-top: 3rem;
      padding-top: 1.5rem;
      border-top: 1px solid var(--border-color);
      font-size: 0.75rem;
      color: var(--text-secondary);
      text-align: center;
    }

    .mobile-footer .repo-link {
      color: var(--text-primary);
      text-decoration: none;
      font-weight: 600;
      margin-bottom: 0.5rem;
      display: inline-flex;
      align-items: center;
      gap: 0.35rem;
      font-size: 0.8rem;
      transition: color 0.2s;
    }

    .mobile-footer .repo-link:hover {
      color: var(--aqua);
    }

    .mobile-footer .coordinator {
      font-weight: 600;
      color: var(--aqua);
      margin-top: 0.1rem;
    }
  }

  @media (max-width: 650px) {
    .content-area {
      padding: 2rem 1.25rem;
    }

    h2 {
      font-size: 1.75rem;
    }

    .subtitle {
      font-size: 0.9rem;
      margin-bottom: 1.75rem;
    }

    .card {
      padding: 1.25rem;
      margin-bottom: 1.25rem;
    }

    .card-header h3 {
      font-size: 1.15rem;
    }

    pre {
      padding: 1rem;
    }

    code {
      font-size: 0.8rem;
    }
  }
</style>
