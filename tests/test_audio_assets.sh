#!/usr/bin/env bash
# Tests para verificar que los assets de audio son válidos y tienen las duraciones esperadas.
# Requiere: sox (soxi)
# Uso: bash tests/test_audio_assets.sh

set -euo pipefail

AUDIO_DIR="assets/audio"
PASS=0
FAIL=0

# Colores
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

pass() { echo -e "${GREEN}PASS${NC} $1"; PASS=$((PASS+1)); }
fail() { echo -e "${RED}FAIL${NC} $1"; FAIL=$((FAIL+1)); }

# -----------------------------------------------------------------------------
# Comprueba que soxi está disponible
# -----------------------------------------------------------------------------
if ! command -v soxi &>/dev/null; then
    echo "ERROR: soxi no encontrado. Instala sox: sudo apt install sox"
    exit 1
fi

# -----------------------------------------------------------------------------
# Definición de cada sonido: archivo | encoding esperado | dur_min_s | dur_max_s
# -----------------------------------------------------------------------------
declare -A MIN_DUR MAX_DUR
MIN_DUR=(
    [rotate.ogg]=0.05
    [land.ogg]=0.10
    [line_clear.ogg]=0.35
    [tetris.ogg]=0.55
    [level_up.ogg]=0.55
    [game_over.ogg]=1.40
)
MAX_DUR=(
    [rotate.ogg]=0.15
    [land.ogg]=0.25
    [line_clear.ogg]=0.65
    [tetris.ogg]=0.75
    [level_up.ogg]=0.80
    [game_over.ogg]=2.00
)

EXPECTED_ENCODING="Vorbis"
EXPECTED_CHANNELS=1

echo "=== Audio asset tests ==="
echo

for FILE in "${!MIN_DUR[@]}"; do
    PATH_OGG="$AUDIO_DIR/$FILE"

    # 1. Existe el fichero
    if [[ ! -f "$PATH_OGG" ]]; then
        fail "$FILE — no existe en $AUDIO_DIR"
        continue
    fi
    pass "$FILE — existe"

    # 2. Es un OGG/Vorbis válido (soxi no falla)
    if ! soxi "$PATH_OGG" &>/dev/null; then
        fail "$FILE — soxi no puede leerlo (fichero corrupto o formato incorrecto)"
        continue
    fi
    pass "$FILE — formato legible por soxi"

    # 3. Encoding es Vorbis
    ENCODING=$(soxi -e "$PATH_OGG" 2>/dev/null)
    if [[ "$ENCODING" == *"$EXPECTED_ENCODING"* ]]; then
        pass "$FILE — encoding Vorbis"
    else
        fail "$FILE — encoding inesperado: '$ENCODING' (esperado: $EXPECTED_ENCODING)"
    fi

    # 4. Canales (mono)
    CHANNELS=$(soxi -c "$PATH_OGG" 2>/dev/null)
    if [[ "$CHANNELS" -eq "$EXPECTED_CHANNELS" ]]; then
        pass "$FILE — mono ($CHANNELS canal)"
    else
        fail "$FILE — canales: $CHANNELS (esperado: $EXPECTED_CHANNELS)"
    fi

    # 5. Sample rate razonable (>= 22050 Hz)
    RATE=$(soxi -r "$PATH_OGG" 2>/dev/null)
    if [[ "$RATE" -ge 22050 ]]; then
        pass "$FILE — sample rate ${RATE} Hz"
    else
        fail "$FILE — sample rate demasiado bajo: ${RATE} Hz (mínimo: 22050)"
    fi

    # 6. Duración dentro del rango esperado
    DURATION=$(soxi -D "$PATH_OGG" 2>/dev/null)
    MIN="${MIN_DUR[$FILE]}"
    MAX="${MAX_DUR[$FILE]}"
    # Comparación con awk (bash no maneja floats)
    IN_RANGE=$(awk "BEGIN { print ($DURATION >= $MIN && $DURATION <= $MAX) ? 1 : 0 }")
    if [[ "$IN_RANGE" -eq 1 ]]; then
        pass "$FILE — duración ${DURATION}s (rango: ${MIN}-${MAX}s)"
    else
        fail "$FILE — duración ${DURATION}s fuera de rango (esperado: ${MIN}-${MAX}s)"
    fi

    # 7. Tamaño de fichero > 1 KB (descarta ficheros vacíos o truncados)
    SIZE=$(stat -c%s "$PATH_OGG")
    if [[ "$SIZE" -gt 1024 ]]; then
        pass "$FILE — tamaño ${SIZE} bytes (> 1 KB)"
    else
        fail "$FILE — fichero demasiado pequeño: ${SIZE} bytes"
    fi

    echo
done

# -----------------------------------------------------------------------------
# Resumen
# -----------------------------------------------------------------------------
TOTAL=$((PASS + FAIL))
echo "=== Resultado: $PASS/$TOTAL tests pasados ==="

if [[ "$FAIL" -gt 0 ]]; then
    exit 1
fi
