// Before edit map data, :set lsp=4 without mode lines.

fn test_map() {
    // position data
    let position = [1.0, 0.0, 0.0];

    // map data
    let map = [];

    // floor: 1
    let height = 0;
    let data = [
        [ 0, 0, 0,],
        [ 0, 0, 0,],
        [ 0, 0, 0,],
    ];
    let floor = #{ height: height, data: data };
    map += floor;

    // floor: 2
    let height = 1;
    let data = [
        [-1,-1,-1,],
        [-1, 3,-1,],
        [-1,-1,-1,],
    ];
    let floor = #{ height: height, data: data };
    map += floor;

    // return result
    let result = #{position: position, map: map};
    result
}
