# vim: lsp=4
def test_map():
    result = []
    # 1st floor
    height = 0
    data = [
        [ 0, 0, 0, 0,],
        [ 0,-1,-1, 0,],
        [ 0, 0, 0, 0,],
    ]
    result.append({'height':height, 'data':data})
    result.append({'height':height, 'data':data})
    return result
