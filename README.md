<!DOCTYPE html>
<html>
<head>
    <title>Maemio: High-Performance In-Memory Database</title>
    <style>
        body {
            font-family: system-ui, -apple-system, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
            color: #333;
            line-height: 1.6;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        .slide {
            background: white;
            border-radius: 8px;
            padding: 40px;
            margin-bottom: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        h1 {
            color: #2563eb;
            font-size: 2.5em;
            margin-bottom: 0.5em;
        }
        h2 {
            color: #1e40af;
            font-size: 2em;
            margin-top: 0;
        }
        .highlight {
            color: #2563eb;
            font-weight: bold;
        }
        .feature-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }
        .feature-card {
            background: #f8fafc;
            padding: 20px;
            border-radius: 6px;
            border: 1px solid #e2e8f0;
            transition: transform 0.2s ease-in-out;
        }
        .feature-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }
        .feature-card h3 {
            color: #1e40af;
            margin-top: 0;
        }
        .code-block {
            background: #1e1e1e;
            color: #d4d4d4;
            padding: 20px;
            border-radius: 6px;
            overflow-x: auto;
            font-family: 'Fira Code', monospace;
            line-height: 1.5;
            font-size: 14px;
        }
        .code-comment { color: #6a9955; }
        .code-keyword { color: #569cd6; }
        .code-string { color: #ce9178; }
        .code-function { color: #dcdcaa; }
        .code-type { color: #4ec9b0; }
        
        .stats {
            display: flex;
            justify-content: space-around;
            margin: 40px 0;
            flex-wrap: wrap;
        }
        .stat-card {
            text-align: center;
            padding: 20px;
            min-width: 200px;
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.05);
            margin: 10px;
            transition: transform 0.2s ease-in-out;
        }
        .stat-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }
        .stat-number {
            font-size: 2.5em;
            color: #2563eb;
            font-weight: bold;
        }
        .stat-label {
            color: #64748b;
            font-size: 1.1em;
            margin-top: 8px;
        }
        .diagram-container {
            margin: 40px auto;
            max-width: 800px;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .architecture-diagram {
            width: 100%;
            height: auto;
        }
        .section-description {
            color: #64748b;
            font-size: 1.1em;
            margin: 20px 0;
            line-height: 1.6;
        }
        .tech-stack {
            display: flex;
            flex-wrap: wrap;
            gap: 15px;
            margin: 20px 0;
        }
        .tech-item {
            background: #f1f5f9;
            padding: 8px 16px;
            border-radius: 20px;
            font-size: 0.9em;
            color: #1e40af;
            font-weight: 500;
        }
    </style>
</head>
<body>
    <div class="container">
        <!-- Title Slide -->
        <div class="slide">
            <h1>Maemio</h1>
            <h2>High-Performance In-Memory Database</h2>
            <p class="section-description">
                A modern, high-throughput transactional database system built in Rust, 
                implementing <a href="https://dl.acm.org/doi/10.1145/3035918.3064015" target="_blank" style="color: #2563eb; text-decoration: none; border-bottom: 1px dotted #2563eb;">Cicada's innovative design principles</a> for exceptional performance 
                and reliability.
            </p>
            <div class="tech-stack">
                <span class="tech-item">Rust</span>
                <span class="tech-item">MVCC</span>
                <span class="tech-item">In-Memory</span>
                <span class="tech-item">Serializable</span>
                <span class="tech-item">Multi-Core</span>
            </div>
        </div>

        <!-- Architecture Overview -->
        <div class="slide">
            <h2>System Architecture and Storage Model</h2>
            <div class="diagram-container">
                <svg viewBox="0 0 800 600" class="architecture-diagram">
                    <!-- SVG content from storage-architecture artifact -->
                    <!-- Background -->
                    <rect width="800" height="600" fill="#ffffff"/>
                    
                    <!-- Client Layer -->
                    <g transform="translate(300,20)">
                        <rect width="200" height="60" rx="10" fill="#2563eb" opacity="0.9"/>
                        <text x="100" y="35" text-anchor="middle" fill="white" font-size="16">Client Applications</text>
                    </g>

                    <!-- Transaction Layer -->
                    <g transform="translate(300,100)">
                        <rect width="200" height="80" rx="10" fill="#1e40af" opacity="0.9"/>
                        <text x="100" y="35" text-anchor="middle" fill="white" font-size="16">Transaction Manager</text>
                        <text x="100" y="55" text-anchor="middle" fill="white" font-size="12">MVCC + Serializable Isolation</text>
                    </g>

                    <!-- Index Layer -->
                    <g transform="translate(50,220)">
                        <rect width="700" height="80" rx="10" fill="#3b82f6" opacity="0.9"/>
                        <text x="350" y="35" text-anchor="middle" fill="white" font-size="16">Index Layer</text>
                        <g transform="translate(20,45)">
                            <text x="120" y="0" text-anchor="middle" fill="white" font-size="12">B-tree Index</text>
                            <text x="350" y="0" text-anchor="middle" fill="white" font-size="12">Hash Index</text>
                            <text x="550" y="0" text-anchor="middle" fill="white" font-size="12">Key Types: Int, String, Bytes</text>
                        </g>
                    </g>

                    <!-- Storage Layer -->
                    <g transform="translate(50,320)">
                        <rect width="700" height="200" rx="10" fill="#1d4ed8" opacity="0.8"/>
                        <text x="350" y="35" text-anchor="middle" fill="white" font-size="16">Storage Layer (Non-Relational)</text>
                        
                        <!-- Record Structure -->
                        <g transform="translate(50,60)">
                            <rect width="180" height="120" rx="5" fill="white" opacity="0.9"/>
                            <text x="90" y="25" text-anchor="middle" font-size="14">Record Structure</text>
                            <text x="20" y="50" font-size="12">• Record ID (u64)</text>
                            <text x="20" y="70" font-size="12">• Version List</text>
                            <text x="20" y="90" font-size="12">• Raw Data (Vec&lt;u8&gt;)</text>
                            <text x="20" y="110" font-size="12">• Timestamps</text>
                        </g>

                        <!-- Version Management -->
                        <g transform="translate(260,60)">
                            <rect width="180" height="120" rx="5" fill="white" opacity="0.9"/>
                            <text x="90" y="25" text-anchor="middle" font-size="14">Version Control</text>
                            <text x="20" y="50" font-size="12">• Multi-Version</text>
                            <text x="20" y="70" font-size="12">• Best-effort Inlining</text>
                            <text x="20" y="90" font-size="12">• Version Chain</text>
                            <text x="20" y="110" font-size="12">• Garbage Collection</text>
                        </g>

                        <!-- Key-Value Nature -->
                        <g transform="translate(470,60)">
                            <rect width="180" height="120" rx="5" fill="white" opacity="0.9"/>
                            <text x="90" y="25" text-anchor="middle" font-size="14">Data Model</text>
                            <text x="20" y="50" font-size="12">• Key-Value Store</text>
                            <text x="20" y="70" font-size="12">• Schema-less</text>
                            <text x="20" y="90" font-size="12">• Binary Values</text>
                            <text x="20" y="110" font-size="12">• No Native Joins</text>
                        </g>
                    </g>

                    <!-- Connection Lines -->
                    <g stroke="#94a3b8" stroke-width="2">
                        <path d="M400,80 L400,100" fill="none"/>
                        <path d="M400,180 L400,220" fill="none"/>
                        <path d="M400,300 L400,320" fill="none"/>
                    </g>

                    <!-- Missing Features Note -->
                    <g transform="translate(50,540)">
                        <text x="0" y="0" font-size="14" fill="#64748b">Missing Relational Features:</text>
                        <text x="0" y="20" font-size="12" fill="#64748b">• Schema Management  • SQL Processing  • Join Operations  • Referential Integrity  • Complex Queries</text>
                    </g>
                </svg>
            </div>
            <p class="section-description">
                Maemio is designed as a high-performance non-relational database, operating as a transactional 
                key-value store with multi-version concurrency control. The storage layer handles raw binary data, 
                making it flexible for various data types while maintaining ACID guarantees. While it could serve 
                as a foundation for a relational system, it currently lacks native support for schemas, SQL, 
                and relational operations.
            </p>
        </div>

        <!-- Transaction Flow -->
        <div class="slide">
            <h2>Transaction Processing Flow</h2>
            <div class="diagram-container">
                <svg viewBox="0 0 800 400" class="architecture-diagram">
                    <!-- Background -->
                    <rect width="800" height="400" fill="#ffffff"/>
                    
                    <!-- Flow Steps -->
                    <g transform="translate(50,50)">
                        <!-- Begin -->
                        <rect width="120" height="60" rx="8" fill="#2563eb" opacity="0.9"/>
                        <text x="60" y="35" text-anchor="middle" fill="white" font-size="14">Begin TX</text>
                        
                        <!-- Read Phase -->
                        <g transform="translate(170,0)">
                            <rect width="120" height="60" rx="8" fill="#3b82f6" opacity="0.9"/>
                            <text x="60" y="35" text-anchor="middle" fill="white" font-size="14">Read Phase</text>
                        </g>
                        
                        <!-- Validation -->
                        <g transform="translate(340,0)">
                            <rect width="120" height="60" rx="8" fill="#1e40af" opacity="0.9"/>
                            <text x="60" y="35" text-anchor="middle" fill="white" font-size="14">Validation</text>
                        </g>
                        
                        <!-- Write Phase -->
                        <g transform="translate(510,0)">
                            <rect width="120" height="60" rx="8" fill="#2563eb" opacity="0.9"/>
                            <text x="60" y="35" text-anchor="middle" fill="white" font-size="14">Write Phase</text>
                        </g>

                        <!-- Success/Fail Outcomes -->
                        <g transform="translate(510,120)">
                            <rect width="120" height="60" rx="8" fill="#16a34a" opacity="0.9"/>
                            <text x="60" y="35" text-anchor="middle" fill="white" font-size="14">Commit</text>
                        </g>
                        
                        <g transform="translate(340,120)">
                            <rect width="120" height="60" rx="8" fill="#dc2626" opacity="0.9"/>
                            <text x="60" y="35" text-anchor="middle" fill="white" font-size="14">Abort & Retry</text>
                        </g>

                        <!-- Arrows -->
                        <g stroke="#94a3b8" stroke-width="2">
                            <path d="M120,30 L170,30" fill="none" marker-end="url(#arrowhead)"/>
                            <path d="M290,30 L340,30" fill="none" marker-end="url(#arrowhead)"/>
                            <path d="M460,30 L510,30" fill="none" marker-end="url(#arrowhead)"/>
                            <path d="M570,60 L570,120" fill="none" marker-end="url(#arrowhead)"/>
                            <path d="M400,60 L400,120" fill="none" marker-end="url(#arrowhead)"/>
                        </g>
                    </g>

                    <!-- Arrow Marker -->
                    <defs>
                        <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                            <polygon points="0 0, 10 3.5, 0 7" fill="#94a3b8"/>
                        </marker>
                    </defs>
                </svg>
            </div>
            <p class="section-description">
                Optimistic multi-version concurrency control enables high throughput 
                while maintaining serializable isolation. The validation phase ensures 
                consistency while the contention manager optimizes performance under load.
            </p>
        </div>

        <!-- Key Features -->
        <div class="slide">
            <h2>Key Features</h2>
            <div class="feature-grid">
                <div class="feature-card">
                    <h3>Multi-Version Concurrency</h3>
                    <p>Optimistic multi-version concurrency control with best-effort version 
                    inlining reduces conflicts and improves read performance.</p>
                </div>
                <div class="feature-card">
                    <h3>Distributed Clock Design</h3>
                    <p>Scalable timestamp allocation using loosely synchronized clocks 
                    enables millions of transactions per second.</p>
                </div>
                <div class="feature-card">
                    <h3>Intelligent Contention Management</h3>
                    <p>Adaptive backoff with hill climbing algorithm automatically 
                    optimizes throughput under varying load conditions.</p>
                </div>
                <div class="feature-card">
                    <h3>Efficient Memory Usage</h3>
                    <p>Rapid garbage collection and best-effort inlining keep memory 
                    footprint minimal while maintaining high performance.</p>
                </div>
            </div>
        </div>

        <!-- Code Examples -->
        <div class="slide">
            <h2>Simple to Use</h2>
            <div class="code-block">
<span class="code-comment">// Initialize database with custom configuration</span>
<span class="code-keyword">let</span> config = <span class="code-type">MaemioConfig</span> {
    thread_count: 4,
    gc_interval: 20,
    clock_sync_interval: 200,
    initial_index_capacity: 2048,
};

<span class="code-keyword">let</span> db = <span class="code-type">Maemio</span>::<span class="code-function">with_config</span>(config)?;
db.<span class="code-function">start_maintenance</span>()?;

<span class="code-comment">// Create indexes</span>
db.<span class="code-function">create_index</span>(1, <span class="code-string">"my_index"</span>, <span class="code-type">IndexType</span>::BTree)?;

<span class="code-comment">// Execute transaction with automatic retry</span>
db.<span class="code-function">execute</span>(thread_id, |tx| {
    <span class="code-comment">// Read existing data</span>
    <span class="code-keyword">let</span> version = tx.<span class="code-function">read</span>(record_id)?;
    
    <span class="code-comment">// Write new data</span>
    tx.<span class="code-function">write</span>(record_id, new_data)?;
    
    <span class="code-comment">// Update index</span>
    <span class="code-keyword">let</span> index = db.<span class="code-function">index_manager</span>()
        .<span class="code-function">get_index</span>(1, <span class="code-string">"my_index"</span>)?;
    index.<span class="code-function">insert</span>(key, record_id, tx.<span class="code-function">get_timestamp</span>())?;
    
    Ok(())
})?;

<span class="code-comment">// Clean shutdown</span>
db.<span class="code-function">shutdown</span>()?;
