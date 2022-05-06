print("test");
// map data
position.xyz = [0.0, 0.0, 0.0];

// 1st floor
let floor = Floor::new();
let height = 0;
let data = 
[
  [ 0, 0, 0],
  [ 0,-1, 0],
  [ 0, 0, 0],
];
floor.height = height;
floor.data = data;
map.push(floor);
