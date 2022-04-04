## O projekcie:
Niniejszy niewielki projekt stanowi 

## Jump start:
W katalogu '**bin**' znajdują się prekompilowane programy (Windows x86-64, Linux x86-64).
Program korzysta z plików konfiguracyjnych .json, aby wykonać pełne wywołanie przykładowej symulacji użyj: '**./fluid_simulation config.json**'.
Dołączony plik konfiguracyjny '**config.json**' zawiera następującje klucze (wszystkie klucze są wymagane):
  - mass_distr_file_path - ścieżka do pliku graficznego z początkowym rozkładem masy (obsługiwane są popularne formaty graficzne jak .bmp, .png, .jpeg ...),
  - output_directory_path - ścieżka do folderu na pliki wynikowe (kolejne klatki symulacji zapisane w formacie .png),
  - frames_number - liczba generowanych klatek,
  - simulation_factor - mnożnik szybkości symulacji,
  - target_resolution - docelowa rozdzielczość generowanych klatek (obsługiwane rozdzielczości: 480, 720, 1080, 1440, 2160),
  - flow_field_scale - skalowanie pola wektorowego (aby otrzymać "gęste" pole wektorowe (z dużą ilościa małych wirów) zmniejsz ten parametr),
  - dynamize_flow_field - randomizacja pola wektorowego w trakcie symulacji,
  - randomize_flow_field - randomizacja początkowego stanu pola wektorowego

## Efekt działania programu:
![](https://github.com/Michal-Szczygiel/fluid_simulation/blob/main/fluid_sim.gif)

Początkowy rozkład masy pochodzi z pliku graficznego (powyżej bitmapa pieczołowicie przygotowana w MS Paint)


![](https://github.com/Michal-Szczygiel/fluid_simulation/blob/main/fluid_sim_2.gif)

Początkowym rozkładem masy może być dosłownie cokolwiek 🎲
