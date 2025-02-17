<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg xmlns="http://www.w3.org/2000/svg" width="600" height="200">
  <style>
    .box { fill: #2563eb; stroke: #1e40af; stroke-width: 2; }
    .text { font-family: Arial, sans-serif; font-size: 14px; fill: white; text-anchor: middle; }
  </style>
  
  <!-- Begin TX -->
  <rect x="10" y="70" width="100" height="50" class="box" rx="5"/>
  <text x="60" y="100" class="text">Begin TX</text>
  
  <!-- Arrow 1 -->
  <line x1="110" y1="95" x2="170" y2="95" stroke="#94a3b8" stroke-width="2" marker-end="url(#arrowhead)"/>
  
  <!-- Read Phase -->
  <rect x="170" y="70" width="100" height="50" class="box" rx="5"/>
  <text x="220" y="100" class="text">Read Phase</text>
  
  <!-- Arrow 2 -->
  <line x1="270" y1="95" x2="330" y2="95" stroke="#94a3b8" stroke-width="2" marker-end="url(#arrowhead)"/>
  
  <!-- Validation -->
  <rect x="330" y="70" width="100" height="50" class="box" rx="5"/>
  <text x="380" y="100" class="text">Validation</text>
  
  <!-- Arrow 3 -->
  <line x1="430" y1="95" x2="490" y2="95" stroke="#94a3b8" stroke-width="2" marker-end="url(#arrowhead)"/>
  
  <!-- Write Phase -->
  <rect x="490" y="70" width="100" height="50" class="box" rx="5"/>
  <text x="540" y="100" class="text">Write Phase</text>
  
  <!-- Outcomes -->
  <rect x="330" y="140" width="100" height="40" fill="#dc2626" rx="5"/>
  <text x="380" y="165" class="text">Abort</text>
  
  <rect x="490" y="140" width="100" height="40" fill="#16a34a" rx="5"/>
  <text x="540" y="165" class="text">Commit</text>
  
  <!-- Arrowhead marker -->
  <defs>
    <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
      <polygon points="0 0, 10 3.5, 0 7" fill="#94a3b8"/>
    </marker>
  </defs>
</svg>
