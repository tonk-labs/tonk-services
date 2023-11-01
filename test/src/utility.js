function i32ToHexTwosComplement(i32) {
    if (i32 >= 0) {
      // For non-negative numbers, direct conversion to hex string
      return "0x" + i32.toString(16);
    } else {
      // For negative numbers, convert to two's complement positive value
      const twosComplement = ((~Math.abs(i32) + 1) & 0xFFFF).toString(16);
      return "0x" + twosComplement;
    }
  }

function hexTwosComplementToI32(hex) {
  const value = parseInt(hex.replace("0x", ""), 16);
  if (!isNaN(value) && value >= 0 && value < 0x10000) {
    if ((value & 0x8000) !== 0) {
      return -((~value + 1) & 0xFFFF);
    } else {
      return value;
    }
  } else {
    return 0x7FFFFFFF;
  }
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

  function getRandomCoordinateAtDistance(origin, distance) {
    // The six possible directions to move in a hex grid
    const directions = [
      { q: 1, r: -1, s: 0 }, { q: 1, r: 0, s: -1 }, { q: 0, r: 1, s: -1 },
      { q: -1, r: 1, s: 0 }, { q: -1, r: 0, s: 1 }, { q: 0, r: -1, s: 1 }
    ];
  
    function addCoordinates(coord1, coord2) {
      return {
        q: coord1.q + coord2.q,
        r: coord1.r + coord2.r,
        s: coord1.s + coord2.s
      };
    }
  
    function scaleCoordinate(coord, scale) {
      return {
        q: coord.q * scale,
        r: coord.r * scale,
        s: coord.s * scale
      };
    }
  
    // Randomly select a direction and scale it by the distance
    const direction = directions[Math.floor(Math.random() * directions.length)];
    const randomDirectionScaled = scaleCoordinate(direction, distance);
  
    // Add the randomly selected direction to the origin
    return addCoordinates(origin, randomDirectionScaled);
  }
  
  function cubeFromHex(q,r,s) {
    return new Cube(
        hexTwosComplementToI32(q),
        hexTwosComplementToI32(r),
        hexTwosComplementToI32(s)
    );
  }
  
  class Cube {
    constructor(q,r,s) {
      this.q = q;
      this.r = r;
      this.s = s;
    }

    add(other) {
      return new Cube([
        '',
        this.q + other.q,
        this.r + other.r,
        this.s + other.s
      ]);
    }
  
    subtract(other) {
      return new Cube([
        '',
        this.q - other.q,
        this.r - other.r,
        this.s - other.s
      ]);
    }
  
    distance(other) {
      const vec = this.subtract(other);
      return (Math.abs(vec.q) + Math.abs(vec.r) + Math.abs(vec.s)) / 2;
    }
  }


  module.exports = {
    i32ToHexTwosComplement,
    hexTwosComplementToI32,
    getRandomCoordinateAtDistance,
    Cube,
    cubeFromHex,
    sleep
  }