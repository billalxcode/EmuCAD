# 74 - RF Amplifier

Dokumen ini mencatat model behavioral untuk 4 entry RF amplifier 3-pin.
Model memakai sumber tegangan dikendalikan tegangan (VCVS) dengan
input resistance eksplisit.

## 1. Persamaan

```
Vout = gain * Vin
Iin = Vin / input_resistance
```

Pin 0 adalah input, pin 1 output, dan pin 2 ground. Output adalah
sumber tegangan ideal terhadap ground. `input_resistance` default
1e6 ohm.

| Entry | Gain default |
|---|---:|
| RF amplifier | 10 |
| low noise amplifier | 100 |
| power amplifier | 2 |
| variable gain amplifier | 1 |

Model bekerja pada analisis DC dan AC karena hanya memakai elemen
linier. Variabel gain amplifier belum memiliki pin kontrol; nilai
`gain` diatur sebagai parameter.

## 2. Batas

- Tidak ada S-parameter, input/output matching, atau return loss.
- Tidak ada bandwidth, noise figure, compression, atau distorsi.
- Tidak ada supply power, efisiensi, atau thermal model.
- Output ideal tidak memiliki output resistance atau current limit.
- Variable gain amplifier tidak memiliki pin kontrol gain.
- Model ini bukan mixer, filter, antenna, atau coupler.

## 3. RF yang Belum Didukung

Empat entry mixer/modulator, empat filter, tiga antenna, dan tiga
coupler/splitter/isolator tetap unsupported. Alasannya:

- Mixer membutuhkan dua input dan pergeseran frekuensi.
- Filter dua port membutuhkan pin input, output, dan ground.
- Antenna membutuhkan model radiasi dan impedansi terhadap
  frekuensi.
- Coupler dan splitter membutuhkan tiga atau empat port dengan
  distribusi daya.

Model RF penuh memerlukan data S-parameter dan solver dua-port.
Tanpa data itu, model generik akan menyesatkan.

## 4. Referensi

- Amplifier, penguatan sinyal dan definisi gain:
  https://en.wikipedia.org/wiki/Amplifier
- Low-noise amplifier, trade-off noise dan gain:
  https://en.wikipedia.org/wiki/Low-noise_amplifier
- RF power amplifier, gain, bandwidth, linearity, dan matching:
  https://en.wikipedia.org/wiki/RF_power_amplifier
- Variable-gain amplifier, gain yang dikendalikan tegangan:
  https://en.wikipedia.org/wiki/Variable-gain_amplifier
- Scattering parameters, dasar model RF dua-port:
  https://en.wikipedia.org/wiki/Scattering_parameters

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 5. Batas Umum

- Belum ada reference vector atau status `calibrated`.
- Belum ada analisis noise dan distortion untuk blok RF.
- Belum ada model matching network.
- Entry RF selain amplifier harus gagal eksplisit di netlist.
