/*
 *  Copyright (c) 2023 Samsung Electronics Co., Ltd All Rights Reserved
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the language governing permissions and
 *  limitations under the License
 */

#include <islet.h>

#include <iostream>
#include <string>
#include <cstring>

#define BUFFER_SIZE 4096

using byte = unsigned char;

static const char CLAIM_TITLE_USER_DATA[] = "User data";
static const char CLAIM_TITLE_PLATFORM_PROFILE[] = "Profile";

bool attestation_test() {
  byte report[BUFFER_SIZE];
  byte claims[BUFFER_SIZE];
  byte value[BUFFER_SIZE];
  int report_len = 0;
  int claims_len = 0;
  int value_len = 0;

  memset(report, 0, sizeof(report));
  memset(claims, 0, sizeof(claims));
  memset(value, 0, sizeof(value));

  // -- Attest -- //
  std::string user_data("User Custom data");
  if (islet_attest((const byte*)user_data.c_str(), user_data.size(), report, &report_len))
    return false;

  // -- Verify -- //
  if (islet_verify(report, report_len, claims, &claims_len))
    return false;

  islet_print_claims(claims, claims_len);

  // -- Parse -- //
  if (islet_parse(CLAIM_TITLE_USER_DATA, claims, claims_len, value, &value_len))
    return false;

  printf("Claim[User data]: %s\n", (char*) value);

  memset(value, 0, sizeof(value));
  if (islet_parse(CLAIM_TITLE_PLATFORM_PROFILE, claims, claims_len, value, &value_len))
    return false;

  printf("Claim[Platform  profile]: %s\n", (char*) value);

  return true;
}

bool sealing_test() {
  byte sealed[BUFFER_SIZE];
  byte unsealed[BUFFER_SIZE];

  int sealed_len = 0;
  int unsealed_len = 0;

  memset(sealed, 0, sizeof(sealed));
  memset(unsealed, 0, sizeof(unsealed));

  // -- Seal -- //
  std::string plaintext(
	"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nunc ornare lacinia accumsan. Nam eleifend vel nisl et"
	"commodo. Quisque in tortor non risus dictum varius. Curabitur pulvinar tellus vitae sapien gravida dapibus. Ut nec "
	"imperdiet sem, eu ornare turpis. Donec a lectus vitae enim malesuada aliquam sed ac turpis. Mauris turpis massa, mollis"
	"et ex vitae, tempor gravida odio. Maecenas erat urna, laoreet et ornare auctor, faucibus nec elit. In luctus turpis "
	"sapien, vel posuere libero pulvinar et. Donec maximus sollicitudin condimentum. Mauris condimentum ex vel purus "
	"scelerisque faucibus. Donec dapibus viverra massa ut iaculis."

	"Maecenas eget sollicitudin lorem. Aenean euismod ultricies dui quis fringilla. Pellentesque sit amet dapibus metus. "
	"Vivamus tincidunt convallis lectus eget lacinia. Aliquam ac nisl vel erat pulvinar accumsan. Aliquam ut ante id nunc "
	"molestie rutrum. Pellentesque facilisis venenatis erat, ac ornare elit posuere in. Integer porttitor sit amet tortor at"
	"lobortis. Morbi imperdiet rutrum metus sed malesuada. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices "
	"posuere cubilia curae; Vestibulum lacinia eu justo nec auctor. Vivamus suscipit a erat in ultricies. Curabitur sit amet"
	"egestas turpis. Sed semper nunc at diam varius, in congue nisl pretium. Proin aliquam magna mi."

	"Ut imperdiet diam ut nisi consequat tincidunt. Sed imperdiet purus vel fermentum dignissim. Etiam at cursus libero. In "
	"leo metus, sagittis at dictum vel, mattis quis quam. Pellentesque in erat purus. Suspendisse in pretium urna, sed "
	"tincidunt felis. Sed dapibus sed ipsum ut mattis. Pellentesque iaculis, dui eget congue hendrerit, velit est "
	"sollicitudin leo, ac ullamcorper diam ligula quis felis. Etiam fermentum magna quis enim pretium, sed rhoncus metus "
	"dignissim."

	"Integer dignissim hendrerit enim, nec blandit massa. Aliquam porttitor dolor vel congue commodo. Donec maximus dui non "
	"neque congue, et aliquet odio pharetra. Integer varius magna vitae dolor efficitur aliquam. Aenean suscipit quam et "
	"lectus tincidunt congue. Aliquam vitae libero dolor. Class aptent taciti sociosqu ad litora torquent per conubia "
	"nostra, per inceptos himenaeos."

	"Donec et ultrices diam, vitae vulputate ligula. Fusce tempor pellentesque commodo. Fusce quam eros, ultrices quis nibh "
	"in, fermentum laoreet tortor. Nam eget tortor et purus dignissim placerat. Etiam eu tellus et leo imperdiet tempor et "
	"porttitor diam. Etiam risus enim, viverra non euismod at, eleifend eu elit. Mauris posuere est lacus, at interdum felis"
	"semper in. Aliquam varius euismod velit, eu iaculis felis malesuada et. Fusce laoreet ac est a euismod. Phasellus vel "
	"sapien dolor. Maecenas vehicula lorem ac orci luctus, vel sodales libero sollicitudin. Nunc id orci mattis, rutrum nunc"
	"id, luctus mauris. Sed sem arcu, pretium quis pellentesque sed, bibendum vel tellus. Integer dapibus pretium ligula, at"
	"rhoncus ante iaculis vel. Proin feugiat enim ut diam mollis, at vestibulum libero vestibulum.");

  if (islet_seal((const byte*)plaintext.c_str(), plaintext.size(), sealed, &sealed_len))
    return false;

  // -- Unseal -- //
  if (islet_unseal(sealed, sealed_len, unsealed, &unsealed_len))
    return false;

  const std::string unsealed_str(unsealed, unsealed + unsealed_len);
  if (plaintext != unsealed_str)
	return false;

  printf("Success sealing round trip.\n");

  return true;
}

int main() {
  bool rv = attestation_test();
  printf("Attestation test %s.\n", (rv ? "succeeded" : "failed"));
  if (!rv)
    return -1;

  rv = sealing_test();
  printf("Sealing test %s.\n", (rv ? "succeeded" : "failed"));

  return 0;
}
