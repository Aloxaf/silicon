// https://www.cnblogs.com/cpuimage/p/5291660.html
#include <math.h>
#include <stdio.h>
#include <stdlib.h>

#if defined _WIN32 || defined __CYGWIN__
#ifdef BUILDING_DLL
#ifdef __GNUC__
#define DLL_PUBLIC __attribute__((dllexport))
#else
#define DLL_PUBLIC                                                             \
  __declspec(                                                                  \
      dllexport) // Note: actually gcc seems to also supports this syntax.
#endif
#else
#ifdef __GNUC__
#define DLL_PUBLIC __attribute__((dllimport))
#else
#define DLL_PUBLIC                                                             \
  __declspec(                                                                  \
      dllimport) // Note: actually gcc seems to also supports this syntax.
#endif
#endif
#define DLL_LOCAL
#else
#if __GNUC__ >= 4
#define DLL_PUBLIC __attribute__((visibility("default")))
#define DLL_LOCAL __attribute__((visibility("hidden")))
#else
#define DLL_PUBLIC
#define DLL_LOCAL
#endif
#endif

void CalGaussianCoeff(float sigma, float *a0, float *a1, float *a2, float *a3,
                      float *b1, float *b2, float *cprev, float *cnext) {
  float alpha, lamma, k;

  if (sigma < 0.5f)
    sigma = 0.5f;
  alpha = (float)exp((0.726) * (0.726)) / sigma;
  lamma = (float)exp(-alpha);
  *b2 = (float)exp(-2 * alpha);
  k = (1 - lamma) * (1 - lamma) / (1 + 2 * alpha * lamma - (*b2));
  *a0 = k;
  *a1 = k * (alpha - 1) * lamma;
  *a2 = k * (alpha + 1) * lamma;
  *a3 = -k * (*b2);
  *b1 = -2 * lamma;
  *cprev = (*a0 + *a1) / (1 + *b1 + *b2);
  *cnext = (*a2 + *a3) / (1 + *b1 + *b2);
}

void gaussianHorizontal(unsigned char *bufferPerLine,
                        unsigned char *lpRowInitial, unsigned char *lpColumn,
                        int width, int height, int Channels,
                        float a0a1, float a2a3, float b1b2, float cprev,
                        float cnext) {
  int HeightStep = Channels * height;
  int WidthSubOne = width - 1;
  if (Channels == 3) {
    float prevOut[3];
    prevOut[0] = (lpRowInitial[0] * cprev);
    prevOut[1] = (lpRowInitial[1] * cprev);
    prevOut[2] = (lpRowInitial[2] * cprev);
    for (int x = 0; x < width; ++x) {
      prevOut[0] = ((lpRowInitial[0] * (a0a1)) - (prevOut[0] * (b1b2)));
      prevOut[1] = ((lpRowInitial[1] * (a0a1)) - (prevOut[1] * (b1b2)));
      prevOut[2] = ((lpRowInitial[2] * (a0a1)) - (prevOut[2] * (b1b2)));
      bufferPerLine[0] = prevOut[0];
      bufferPerLine[1] = prevOut[1];
      bufferPerLine[2] = prevOut[2];
      bufferPerLine += Channels;
      lpRowInitial += Channels;
    }
    lpRowInitial -= Channels;
    lpColumn += HeightStep * WidthSubOne;
    bufferPerLine -= Channels;
    prevOut[0] = (lpRowInitial[0] * cnext);
    prevOut[1] = (lpRowInitial[1] * cnext);
    prevOut[2] = (lpRowInitial[2] * cnext);

    for (int x = WidthSubOne; x >= 0; --x) {
      prevOut[0] = ((lpRowInitial[0] * (a2a3)) - (prevOut[0] * (b1b2)));
      prevOut[1] = ((lpRowInitial[1] * (a2a3)) - (prevOut[1] * (b1b2)));
      prevOut[2] = ((lpRowInitial[2] * (a2a3)) - (prevOut[2] * (b1b2)));
      bufferPerLine[0] += prevOut[0];
      bufferPerLine[1] += prevOut[1];
      bufferPerLine[2] += prevOut[2];
      lpColumn[0] = bufferPerLine[0];
      lpColumn[1] = bufferPerLine[1];
      lpColumn[2] = bufferPerLine[2];
      lpRowInitial -= Channels;
      lpColumn -= HeightStep;
      bufferPerLine -= Channels;
    }
  } else if (Channels == 4) {
    float prevOut[4];

    prevOut[0] = (lpRowInitial[0] * cprev);
    prevOut[1] = (lpRowInitial[1] * cprev);
    prevOut[2] = (lpRowInitial[2] * cprev);
    prevOut[3] = (lpRowInitial[3] * cprev);
    for (int x = 0; x < width; ++x) {
      prevOut[0] = ((lpRowInitial[0] * (a0a1)) - (prevOut[0] * (b1b2)));
      prevOut[1] = ((lpRowInitial[1] * (a0a1)) - (prevOut[1] * (b1b2)));
      prevOut[2] = ((lpRowInitial[2] * (a0a1)) - (prevOut[2] * (b1b2)));
      prevOut[3] = ((lpRowInitial[3] * (a0a1)) - (prevOut[3] * (b1b2)));

      bufferPerLine[0] = prevOut[0];
      bufferPerLine[1] = prevOut[1];
      bufferPerLine[2] = prevOut[2];
      bufferPerLine[3] = prevOut[3];
      bufferPerLine += Channels;
      lpRowInitial += Channels;
    }
    lpRowInitial -= Channels;
    lpColumn += HeightStep * WidthSubOne;
    bufferPerLine -= Channels;

    prevOut[0] = (lpRowInitial[0] * cnext);
    prevOut[1] = (lpRowInitial[1] * cnext);
    prevOut[2] = (lpRowInitial[2] * cnext);
    prevOut[3] = (lpRowInitial[3] * cnext);

    for (int x = WidthSubOne; x >= 0; --x) {
      prevOut[0] = ((lpRowInitial[0] * a2a3) - (prevOut[0] * b1b2));
      prevOut[1] = ((lpRowInitial[1] * a2a3) - (prevOut[1] * b1b2));
      prevOut[2] = ((lpRowInitial[2] * a2a3) - (prevOut[2] * b1b2));
      prevOut[3] = ((lpRowInitial[3] * a2a3) - (prevOut[3] * b1b2));
      bufferPerLine[0] += prevOut[0];
      bufferPerLine[1] += prevOut[1];
      bufferPerLine[2] += prevOut[2];
      bufferPerLine[3] += prevOut[3];
      lpColumn[0] = bufferPerLine[0];
      lpColumn[1] = bufferPerLine[1];
      lpColumn[2] = bufferPerLine[2];
      lpColumn[3] = bufferPerLine[3];
      lpRowInitial -= Channels;
      lpColumn -= HeightStep;
      bufferPerLine -= Channels;
    }
  } else if (Channels == 1) {
    float prevOut = (lpRowInitial[0] * cprev);

    for (int x = 0; x < width; ++x) {
      prevOut = ((lpRowInitial[0] * (a0a1)) - (prevOut * (b1b2)));
      bufferPerLine[0] = prevOut;
      bufferPerLine += Channels;
      lpRowInitial += Channels;
    }
    lpRowInitial -= Channels;
    lpColumn += HeightStep * WidthSubOne;
    bufferPerLine -= Channels;

    prevOut = (lpRowInitial[0] * cnext);

    for (int x = WidthSubOne; x >= 0; --x) {
      prevOut = ((lpRowInitial[0] * a2a3) - (prevOut * b1b2));
      bufferPerLine[0] += prevOut;
      lpColumn[0] = bufferPerLine[0];
      lpRowInitial -= Channels;
      lpColumn -= HeightStep;
      bufferPerLine -= Channels;
    }
  }
}

void gaussianVertical(unsigned char *bufferPerLine, unsigned char *lpRowInitial,
                      unsigned char *lpColInitial, int height, int width,
                      int Channels, float a0a1, float a2a3, float b1b2,
                      float cprev, float cnext) {

  int WidthStep = Channels * width;
  int HeightSubOne = height - 1;
  if (Channels == 3) {
    float prevOut[3];
    prevOut[0] = (lpRowInitial[0] * cprev);
    prevOut[1] = (lpRowInitial[1] * cprev);
    prevOut[2] = (lpRowInitial[2] * cprev);

    for (int y = 0; y < height; y++) {
      prevOut[0] = ((lpRowInitial[0] * a0a1) - (prevOut[0] * b1b2));
      prevOut[1] = ((lpRowInitial[1] * a0a1) - (prevOut[1] * b1b2));
      prevOut[2] = ((lpRowInitial[2] * a0a1) - (prevOut[2] * b1b2));
      bufferPerLine[0] = prevOut[0];
      bufferPerLine[1] = prevOut[1];
      bufferPerLine[2] = prevOut[2];
      bufferPerLine += Channels;
      lpRowInitial += Channels;
    }
    lpRowInitial -= Channels;
    bufferPerLine -= Channels;
    lpColInitial += WidthStep * HeightSubOne;
    prevOut[0] = (lpRowInitial[0] * cnext);
    prevOut[1] = (lpRowInitial[1] * cnext);
    prevOut[2] = (lpRowInitial[2] * cnext);
    for (int y = HeightSubOne; y >= 0; y--) {
      prevOut[0] = ((lpRowInitial[0] * a2a3) - (prevOut[0] * b1b2));
      prevOut[1] = ((lpRowInitial[1] * a2a3) - (prevOut[1] * b1b2));
      prevOut[2] = ((lpRowInitial[2] * a2a3) - (prevOut[2] * b1b2));
      bufferPerLine[0] += prevOut[0];
      bufferPerLine[1] += prevOut[1];
      bufferPerLine[2] += prevOut[2];
      lpColInitial[0] = bufferPerLine[0];
      lpColInitial[1] = bufferPerLine[1];
      lpColInitial[2] = bufferPerLine[2];
      lpRowInitial -= Channels;
      lpColInitial -= WidthStep;
      bufferPerLine -= Channels;
    }
  } else if (Channels == 4) {
    float prevOut[4];

    prevOut[0] = (lpRowInitial[0] * cprev);
    prevOut[1] = (lpRowInitial[1] * cprev);
    prevOut[2] = (lpRowInitial[2] * cprev);
    prevOut[3] = (lpRowInitial[3] * cprev);

    for (int y = 0; y < height; y++) {
      prevOut[0] = ((lpRowInitial[0] * a0a1) - (prevOut[0] * b1b2));
      prevOut[1] = ((lpRowInitial[1] * a0a1) - (prevOut[1] * b1b2));
      prevOut[2] = ((lpRowInitial[2] * a0a1) - (prevOut[2] * b1b2));
      prevOut[3] = ((lpRowInitial[3] * a0a1) - (prevOut[3] * b1b2));
      bufferPerLine[0] = prevOut[0];
      bufferPerLine[1] = prevOut[1];
      bufferPerLine[2] = prevOut[2];
      bufferPerLine[3] = prevOut[3];
      bufferPerLine += Channels;
      lpRowInitial += Channels;
    }
    lpRowInitial -= Channels;
    bufferPerLine -= Channels;
    lpColInitial += WidthStep * HeightSubOne;
    prevOut[0] = (lpRowInitial[0] * cnext);
    prevOut[1] = (lpRowInitial[1] * cnext);
    prevOut[2] = (lpRowInitial[2] * cnext);
    prevOut[3] = (lpRowInitial[3] * cnext);
    for (int y = HeightSubOne; y >= 0; y--) {
      prevOut[0] = ((lpRowInitial[0] * a2a3) - (prevOut[0] * b1b2));
      prevOut[1] = ((lpRowInitial[1] * a2a3) - (prevOut[1] * b1b2));
      prevOut[2] = ((lpRowInitial[2] * a2a3) - (prevOut[2] * b1b2));
      prevOut[3] = ((lpRowInitial[3] * a2a3) - (prevOut[3] * b1b2));
      bufferPerLine[0] += prevOut[0];
      bufferPerLine[1] += prevOut[1];
      bufferPerLine[2] += prevOut[2];
      bufferPerLine[3] += prevOut[3];
      lpColInitial[0] = bufferPerLine[0];
      lpColInitial[1] = bufferPerLine[1];
      lpColInitial[2] = bufferPerLine[2];
      lpColInitial[3] = bufferPerLine[3];
      lpRowInitial -= Channels;
      lpColInitial -= WidthStep;
      bufferPerLine -= Channels;
    }
  } else if (Channels == 1) {
    float prevOut = 0;
    prevOut = (lpRowInitial[0] * cprev);
    for (int y = 0; y < height; y++) {
      prevOut = ((lpRowInitial[0] * a0a1) - (prevOut * b1b2));
      bufferPerLine[0] = prevOut;
      bufferPerLine += Channels;
      lpRowInitial += Channels;
    }
    lpRowInitial -= Channels;
    bufferPerLine -= Channels;
    lpColInitial += WidthStep * HeightSubOne;
    prevOut = (lpRowInitial[0] * cnext);
    for (int y = HeightSubOne; y >= 0; y--) {
      prevOut = ((lpRowInitial[0] * a2a3) - (prevOut * b1b2));
      bufferPerLine[0] += prevOut;
      lpColInitial[0] = bufferPerLine[0];
      lpRowInitial -= Channels;
      lpColInitial -= WidthStep;
      bufferPerLine -= Channels;
    }
  }
}

//本人博客:http://tntmonks.cnblogs.com/ 转载请注明出处.
DLL_PUBLIC void GaussianBlurFilter(unsigned char *input,
                                         unsigned char *output, int Width,
                                         int Height, int Stride,
                                         float GaussianSigma) {

  int Channels = Stride / Width;
  float a0, a1, a2, a3, b1, b2, cprev, cnext;

  CalGaussianCoeff(GaussianSigma, &a0, &a1, &a2, &a3, &b1, &b2, &cprev, &cnext);

  float a0a1 = (a0 + a1);
  float a2a3 = (a2 + a3);
  float b1b2 = (b1 + b2);

  int bufferSizePerThread = (Width > Height ? Width : Height) * Channels;
  unsigned char *bufferPerLine = (unsigned char *)malloc(bufferSizePerThread);
  unsigned char *tempData = (unsigned char *)malloc(Height * Stride);
  if (bufferPerLine == NULL || tempData == NULL) {
    if (tempData) {
      free(tempData);
    }
    if (bufferPerLine) {
      free(bufferPerLine);
    }
    return;
  }
  for (int y = 0; y < Height; ++y) {
    unsigned char *lpRowInitial = input + Stride * y;
    unsigned char *lpColInitial = tempData + y * Channels;
    gaussianHorizontal(bufferPerLine, lpRowInitial, lpColInitial, Width, Height,
                       Channels, a0a1, a2a3, b1b2, cprev, cnext);
  }
  int HeightStep = Height * Channels;
  for (int x = 0; x < Width; ++x) {
    unsigned char *lpColInitial = output + x * Channels;
    unsigned char *lpRowInitial = tempData + HeightStep * x;
    gaussianVertical(bufferPerLine, lpRowInitial, lpColInitial, Height, Width,
                     Channels, a0a1, a2a3, b1b2, cprev, cnext);
  }

  free(bufferPerLine);
  free(tempData);
}
